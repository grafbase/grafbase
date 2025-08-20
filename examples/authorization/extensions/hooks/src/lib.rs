use std::collections::HashMap;

use common::AuthContext;
use grafbase_sdk::{
    HooksExtension,
    types::{AuthorizedOperationContext, Configuration, Error, HttpRequestParts},
};

#[derive(HooksExtension)]
struct Hooks;

impl HooksExtension for Hooks {
    fn new(_config: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    fn on_graphql_subgraph_request(
        &mut self,
        ctx: &AuthorizedOperationContext,
        subgraph_name: &str,
        parts: &mut HttpRequestParts,
    ) -> Result<(), Error> {
        // FIXME: simplify with gateway 0.47.2 and SDK 0.22
        let bytes = ctx.authorization_icontext_by_key("my-authorization")?;
        let AuthContext { scopes } = postcard::from_bytes(&bytes).unwrap();

        if let Some(token) = scopes.get(subgraph_name) {
            parts.headers.append("Authorization", token);
        }
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SubgraphTokens {
    // Token by subgraph name
    tokens: HashMap<String, String>,
}
