use grafbase_sdk::{
    AuthorizationExtension, IntoAuthorizeQueryOutput,
    types::{
        AuthenticatedRequestContext, AuthorizationDecisions, AuthorizeQueryOutput, AuthorizedOperationContext,
        Configuration, Error, ErrorResponse, QueryElements, ResponseElements, SubgraphHeaders,
    },
};

#[derive(AuthorizationExtension)]
struct CustomAuthorization;

#[derive(serde::Deserialize)]
struct DeniedIdsArgs {
    ids: Vec<u32>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
struct State {
    denied_ids: Vec<DeniedIds>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
struct DeniedIds {
    query_element_id: u32,
    denied_ids: Vec<u32>,
}

#[serde_with::serde_as]
#[derive(serde::Deserialize)]
struct ResponseArguments<'a> {
    #[serde(borrow)]
    #[serde_as(as = "serde_with::Map<_, _>")]
    fields: Vec<(&'a str, serde_json::Value)>,
}

impl ResponseArguments<'_> {
    fn id_as_u32(&self) -> u32 {
        self.fields.first().and_then(|(_, value)| value.as_u64()).unwrap() as u32
    }
}

impl AuthorizationExtension for CustomAuthorization {
    fn new(_: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    fn authorize_query(
        &mut self,
        ctx: &AuthenticatedRequestContext,
        _headers: &SubgraphHeaders,
        elements: QueryElements<'_>,
    ) -> Result<impl IntoAuthorizeQueryOutput, ErrorResponse> {
        let mut state = State { denied_ids: Vec::new() };
        let mut builder = AuthorizationDecisions::deny_some_builder();
        let error_id = builder.push_error(Error::new("Not authorized, query auth SDK021"));

        for (name, elements) in elements.iter_grouped_by_directive_name() {
            match name {
                "deniedIds" => {
                    for element in elements {
                        let args: DeniedIdsArgs = element.directive_arguments()?;
                        state.denied_ids.push(DeniedIds {
                            query_element_id: element.id().into(),
                            denied_ids: args.ids,
                        });
                    }
                }
                "deny" => {
                    for element in elements {
                        builder.deny_with_error_id(element, error_id);
                    }
                }
                _ => unreachable!(),
            }
        }

        let state = rkyv::api::high::to_bytes_in::<_, rkyv::rancor::Error>(&state, Vec::new()).unwrap();
        Ok(AuthorizeQueryOutput::new(builder.build())
            .state(state)
            .header("token", ctx.token().as_bytes().unwrap_or_default()))
    }

    fn authorize_response(
        &mut self,
        _ctx: &AuthorizedOperationContext,
        state: Vec<u8>,
        elements: ResponseElements<'_>,
    ) -> Result<AuthorizationDecisions, Error> {
        let state = rkyv::access::<ArchivedState, rkyv::rancor::Error>(&state).unwrap();
        let mut builder = AuthorizationDecisions::deny_some_builder();
        let error_id = builder.push_error(Error::new("Not authorized, response auth SDK021"));

        for element in elements {
            if let Some(denied) = state
                .denied_ids
                .iter()
                .find(|denied| denied.query_element_id == u32::from(element.query_element_id()))
            {
                for item in element.items() {
                    let object: ResponseArguments<'_> = item.directive_arguments()?;
                    if denied.denied_ids.contains(&(object.id_as_u32().into())) {
                        builder.deny_with_error_id(item, error_id);
                    }
                }
            }
        }

        Ok(builder.build())
    }
}
