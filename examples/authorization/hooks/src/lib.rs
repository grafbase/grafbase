use std::collections::HashMap;

use grafbase_sdk::{
    HooksExtension,
    types::{AuthorizedOperationContext, Configuration, Error, HttpRequestParts},
};

#[derive(HooksExtension)]
struct Hooks;

impl HooksExtension for Hooks {
    fn new(config: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    fn on_graphql_subgraph_request(
        &mut self,
        ctx: &AuthorizedOperationContext,
        subgraph_name: &str,
        parts: &mut HttpRequestParts,
    ) -> Result<(), Error> {
        let context: SubgraphTokens = serde_json::from_slice(&ctx.authorization_context()?).unwrap();
        if let Some(token) = context.tokens.get(subgraph_name) {
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

