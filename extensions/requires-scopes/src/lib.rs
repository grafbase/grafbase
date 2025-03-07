use std::borrow::Cow;

use grafbase_sdk::{
    AuthorizationExtension, Error,
    host::AuthorizationContext,
    types::{AuthorizationDecisions, Configuration, ErrorResponse, QueryElements},
};

#[derive(AuthorizationExtension)]
struct RequiresScopes;

#[derive(serde::Deserialize)]
struct Claims<'a> {
    #[serde(borrow)]
    scope: Cow<'a, str>,
}

#[derive(serde::Deserialize)]
struct DirectiveArguments<'a> {
    #[serde(borrow)]
    scopes: Vec<Vec<Scope<'a>>>,
}

#[derive(serde::Deserialize)]
struct Scope<'a>(#[serde(borrow)] Cow<'a, str>);

impl Scope<'_> {
    fn as_str(&self) -> &str {
        self.0.as_ref()
    }
}

impl AuthorizationExtension for RequiresScopes {
    fn new(_config: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    fn authorize_query(
        &mut self,
        ctx: AuthorizationContext,
        elements: QueryElements<'_>,
    ) -> Result<AuthorizationDecisions, ErrorResponse> {
        let Some(bytes) = ctx.token().into_bytes() else {
            // Anonymous user.
            return Ok(AuthorizationDecisions::deny_all("Not authorized"));
        };
        let Ok(Claims { scope }) = serde_json::from_slice(&bytes) else {
            // Unsupported token.
            return Ok(AuthorizationDecisions::deny_all("Not authorized: unsupported token."));
        };
        let token_scopes = scope.split(' ').collect::<Vec<_>>();

        let mut builder = AuthorizationDecisions::deny_some_builder();
        let mut lazy_error_id = None;

        for element in elements {
            let DirectiveArguments { scopes } = element.arguments::<DirectiveArguments>()?;
            let has_matching_scopes = scopes
                .iter()
                .any(|scopes| scopes.iter().all(|scope| token_scopes.contains(&scope.as_str())));

            if !has_matching_scopes {
                let error_id =
                    *lazy_error_id.get_or_insert_with(|| builder.push_error("Not authorized: insufficient scopes"));
                // We re-use the same GraphQL error here to avoid sending duplicate data back to
                // the gateway. The GraphQL response will have an individual error for each element
                // however.
                builder.deny_with_error_id(element, error_id);
            }
        }

        Ok(builder.build())
    }
}
