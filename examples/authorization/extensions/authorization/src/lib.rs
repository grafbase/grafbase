mod directive;
mod state;

use std::collections::HashSet;

use grafbase_sdk::{
    AuthorizationExtension, IntoQueryAuthorization,
    host_io::{self, http::HttpRequest},
    types::{
        AuthorizationDecisions, Configuration, DirectiveSite, Error, ErrorResponse, QueryElements, SubgraphHeaders,
        Token,
    },
};

use directive::*;
use itertools::Itertools;
use state::*;

#[derive(AuthorizationExtension)]
struct MyAuthorization {
    config: Config,
}

#[derive(serde::Deserialize)]
struct Config {
    auth_service_url: String,
}

impl AuthorizationExtension for MyAuthorization {
    fn new(config: Configuration) -> Result<Self, Error> {
        Ok(Self {
            config: config.deserialize()?,
        })
    }

    fn authorize_query(
        &mut self,
        subgraph_headers: &mut SubgraphHeaders,
        token: Token,
        elements: QueryElements<'_>,
    ) -> Result<impl IntoQueryAuthorization, ErrorResponse> {
        // Deserialize the token that has been generated by our authentication extension.
        // We expect it to be present and properly serialized, if not we stop the request
        // processing.
        let Some(common::Token { current_user_id }) =
            token.as_bytes().and_then(|bytes| postcard::from_bytes(bytes).ok())
        else {
            return Err(ErrorResponse::internal_server_error());
        };

        // Builder will keep track of all of our authorization decisions. There are two simpler
        // variants `AuthorizationDecisions::grant_all()` and `AuthorizationDecisions::deny_all()`
        // for simpler cases.
        let mut builder = AuthorizationDecisions::deny_some_builder();
        let mut lazy_error_id = None;

        // We accumulate all the scopes we need
        let mut required_jwt_scopes_accumulator = HashSet::new();
        // List of authorized user ids we lazily retrieve from the auth-service.
        let mut authorized_user_ids = None;

        let mut state = State::default();

        // Each element represents an object, field, enum, etc. within the query that was decorated
        // with one of our directives.
        for (directive_name, elements) in elements.iter_grouped_by_directive_name() {
            match directive_name {
                "jwtScope" => {
                    for element in elements {
                        let JwtScopeArguments { scopes } = element.directive_arguments::<JwtScopeArguments>()?;
                        required_jwt_scopes_accumulator.extend(scopes);
                    }
                }
                "accessControl" => {
                    for element in elements {
                        match element.directive_site() {
                            DirectiveSite::Object(object) => match object.name() {
                                "Account" => state.denied_ids.push(DeniedIds {
                                    query_element_id: element.id().into(),
                                    authorized_ids: if let Some(ids) = authorized_user_ids.as_ref() {
                                        ids
                                    } else {
                                        authorized_user_ids = Some(self.get_authorized_ids(current_user_id)?);
                                        authorized_user_ids.as_ref().unwrap()
                                    }
                                    .clone(),
                                }),
                                _ => {
                                    return Err(unsupported());
                                }
                            },
                            DirectiveSite::FieldDefinition(field) => {
                                match (field.parent_type_name(), field.name()) {
                                    ("Query", "user") => {
                                        let AccessControlArguments { arguments, .. } = element.directive_arguments()?;
                                        let arguments = arguments.unwrap();
                                        let ids = if let Some(ids) = authorized_user_ids.as_ref() {
                                            ids
                                        } else {
                                            authorized_user_ids = Some(self.get_authorized_ids(current_user_id)?);
                                            authorized_user_ids.as_ref().unwrap()
                                        };
                                        if !ids.contains(&arguments.id_as_u32()) {
                                            let error_id = *lazy_error_id.get_or_insert_with(|| {
                                                builder.push_error("Not authorized: cannot access user")
                                            });
                                            // We re-use the same GraphQL error here to avoid sending duplicate data back to
                                            // the gateway. The GraphQL response will have an individual error for each element
                                            // however.
                                            builder.deny_with_error_id(element, error_id);
                                        }
                                    }
                                    _ => {
                                        return Err(unsupported());
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        // We set the Authorization header with the required scopes for the subgraphs
        let scopes = http::HeaderValue::from_bytes(
            Itertools::intersperse(required_jwt_scopes_accumulator.into_iter(), ",")
                .collect::<String>()
                .as_bytes(),
        )
        .unwrap();
        subgraph_headers.set(http::header::AUTHORIZATION, [scopes]);

        // For a simpler alternative that works with serde, we recommend `postcard`. rkyv has the
        // benefit of proving zero-copy deserialization. And authorize-response may be called
        // multiple times contrary to authorize_query which is called only once if necessary.
        let state = rkyv::api::high::to_bytes_in::<_, rkyv::rancor::Error>(&state, Vec::new()).unwrap();
        Ok((builder.build(), state))
    }

    fn authorize_response(
        &mut self,
        state: Vec<u8>,
        elements: grafbase_sdk::types::ResponseElements<'_>,
    ) -> Result<AuthorizationDecisions, Error> {
        let state = rkyv::access::<ArchivedState, rkyv::rancor::Error>(&state).unwrap();

        let mut builder = AuthorizationDecisions::deny_some_builder();
        let mut lazy_error_id = None;

        // Each element here matches one of the query elements we received in authorize_query. But
        // we only receive them here if and only if the directive requested something from the
        // response, with a `FieldSet` typically.
        for element in elements {
            if let Some(denied) = state
                .denied_ids
                .iter()
                .find(|denied| denied.query_element_id == u32::from(element.query_element_id()))
            {
                // Each item here represents an item within the GraphQL response. So if the query
                // was something like `query { users { secret } }`. We'll only receive one element
                // for `users` field or `User` type, etc. depending on the directive location. But
                // an item for every occurrence of the user within the response.
                for item in element.items() {
                    let AccessControlArguments { fields, .. } = item.directive_arguments()?;
                    let fields = fields.unwrap();
                    if !denied.authorized_ids.contains(&(fields.id_as_u32().into())) {
                        let error_id = *lazy_error_id
                            .get_or_insert_with(|| builder.push_error("Not authorized: cannot access account"));
                        builder.deny_with_error_id(item, error_id);
                    }
                }
            }
        }

        Ok(builder.build())
    }
}

impl MyAuthorization {
    fn get_authorized_ids(&self, current_user_id: u32) -> Result<Vec<u32>, ErrorResponse> {
        host_io::http::execute(
            &HttpRequest::post(
                format!("{}/authorized-users", self.config.auth_service_url)
                    .parse()
                    .unwrap(),
            )
            .json(&serde_json::json!({"current_user_id": current_user_id})),
        )
        .map(|response| {
            let AuthorizationResponse { authorized_users } = response.json().unwrap();
            authorized_users
        })
        .map_err(|e| ErrorResponse::unauthorized().with_error(Error::new(e.to_string())))
    }
}
#[derive(serde::Deserialize)]
struct AuthorizationResponse {
    authorized_users: Vec<u32>,
}

fn unsupported() -> ErrorResponse {
    ErrorResponse::internal_server_error().with_error(Error::new("Unsupported"))
}
