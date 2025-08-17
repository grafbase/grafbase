use std::collections::HashMap;

use grafbase_sdk::{
    AuthorizationExtension, IntoAuthorizeQueryOutput,
    types::{
        AuthenticatedRequestContext, AuthorizationDecisions, AuthorizeQueryOutput, Configuration, Error, ErrorResponse,
        QueryElements, SubgraphHeaders,
    },
};

#[derive(AuthorizationExtension)]
struct Authz;

impl AuthorizationExtension for Authz {
    fn new(config: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    fn authorize_query(
        &mut self,
        ctx: &AuthenticatedRequestContext,
        headers: &SubgraphHeaders,
        elements: QueryElements<'_>,
    ) -> Result<impl IntoAuthorizeQueryOutput, ErrorResponse> {
        let tokens = SubgraphTokens {
            tokens: vec![("products".to_string(), "token".to_string())]
                .into_iter()
                .collect(),
        };
        let context = serde_json::to_vec(&tokens).unwrap();
        Ok(AuthorizeQueryOutput::new(AuthorizationDecisions::grant_all()).context(context))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SubgraphTokens {
    // Token by subgraph name
    tokens: HashMap<String, String>,
}

