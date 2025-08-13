use grafbase_sdk::{
    AuthorizationExtension, IntoAuthorizeQueryOutput,
    host_io::logger::log,
    types::{
        AuthenticatedRequestContext, AuthorizationDecisions, AuthorizeQueryOutput, AuthorizedOperationContext,
        Configuration, Error, ErrorResponse, QueryElements, ResponseElements, SubgraphHeaders,
    },
};

#[derive(AuthorizationExtension)]
struct CustomAuthorization {
    config: Config,
}

#[derive(Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Config {
    context: Option<String>,
    response_error_with_context: bool,
    error_authorization_contexts: Option<Vec<String>>,
}

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
    fn new(config: Configuration) -> Result<Self, Error> {
        let config: Config = config.deserialize().unwrap_or_default();
        Ok(Self { config })
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
                "grant" => {}
                _ => unreachable!(),
            }
        }

        let state = rkyv::api::high::to_bytes_in::<_, rkyv::rancor::Error>(&state, Vec::new()).unwrap();
        let context = if let Some(context) = &self.config.context {
            context.clone().into_bytes()
        } else {
            Vec::new()
        };
        Ok(AuthorizeQueryOutput::new(builder.build())
            .state(state)
            .context(context)
            .header("hooks-context", ctx.hooks_context())
            .header("token", ctx.token().as_bytes().unwrap_or_default()))
    }

    fn authorize_response(
        &mut self,
        ctx: &AuthorizedOperationContext,
        state: Vec<u8>,
        elements: ResponseElements<'_>,
    ) -> Result<AuthorizationDecisions, Error> {
        if self.config.response_error_with_context {
            let authorization_context = self
                .config
                .error_authorization_contexts
                .as_ref()
                .map(|keys| {
                    keys.iter()
                        .map(|key| {
                            ctx.authorization_icontext_by_key(key)
                                .ok()
                                .map(|context| String::from_utf8_lossy(&context).into_owned())
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_else(|| {
                    vec![
                        ctx.authorization_context()
                            .ok()
                            .map(|context| String::from_utf8_lossy(&context).into_owned()),
                    ]
                });
            return Err(Error::new("Failure")
                .extension("token", ctx.token().as_bytes().map(String::from_utf8_lossy))?
                .extension("authorization_context", authorization_context)?
                .extension("hooks_context", String::from_utf8_lossy(&ctx.hooks_context()))?);
        }
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
