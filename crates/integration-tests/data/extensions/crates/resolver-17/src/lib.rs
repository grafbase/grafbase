use std::collections::VecDeque;

use grafbase_sdk::{
    IntoSubscription, ResolverExtension,
    types::{
        Configuration, Error, ResolvedField, Response, SubgraphHeaders, SubgraphSchema, SubscriptionItem, Variables,
    },
};

#[derive(ResolverExtension)]
struct Resolver17 {
    config: Config,
}

// Configuration in the TOML for this extension
#[derive(serde::Deserialize, serde::Serialize)]
struct Config {
    #[serde(default)]
    key: Option<String>,
}

impl ResolverExtension for Resolver17 {
    fn new(_subgraph_schemas: Vec<SubgraphSchema<'_>>, config: Configuration) -> Result<Self, Error> {
        let config: Config = config.deserialize()?;
        Ok(Self { config })
    }

    fn resolve(&mut self, prepared: &[u8], _headers: SubgraphHeaders, variables: Variables) -> Result<Response, Error> {
        // field which must be resolved. The prepared bytes can be customized to store anything you need in the operation cache.
        let field = ResolvedField::try_from(prepared)?;
        let arguments: serde_json::Value = field.arguments(&variables)?;
        let dir_args: serde_json::Value = field.directive().arguments()?;
        Ok(Response::data(serde_json::json!({
            "args": arguments,
            "directive": dir_args,
            "config": &self.config
        })))
    }

    fn resolve_subscription<'s>(
        &'s mut self,
        prepared: &'s [u8],
        _headers: SubgraphHeaders,
        variables: Variables,
    ) -> Result<impl IntoSubscription<'s>, Error> {
        let field = ResolvedField::try_from(prepared)?;
        let arguments: serde_json::Value = field.arguments(&variables)?;
        let dir_args: serde_json::Value = field.directive().arguments()?;
        Ok(Subscription {
            items: vec![
                serde_json::json!({
                    "args": arguments,
                    "directive": dir_args,
                    "config": &self.config
                }),
                serde_json::json!({
                    "message": "This is a test message from the resolver extension.",
                }),
            ]
            .into(),
        })
    }
}

struct Subscription {
    items: VecDeque<serde_json::Value>,
}

impl grafbase_sdk::Subscription for Subscription {
    fn next(&mut self) -> Result<Option<SubscriptionItem>, Error> {
        Ok(self.items.pop_front().map(|item| Response::data(item).into()))
    }
}
