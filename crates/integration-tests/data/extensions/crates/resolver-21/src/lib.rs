use std::collections::VecDeque;

use grafbase_sdk::{
    IntoSubscription, ResolverExtension,
    types::{
        AuthorizedOperationContext, Configuration, Error, ResolvedField, Response, SubgraphHeaders, SubgraphSchema,
        SubscriptionItem, Variables,
    },
};
use serde_json::{Value, json};

#[derive(ResolverExtension)]
struct Resolver17 {
    config: Config,
}

// Configuration in the TOML for this extension
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum Config {
    EchoInput {
        #[serde(default)]
        key: Option<String>,
    },
    EchoContext {
        #[serde(default)]
        authorization_context: Option<Vec<String>>,
    },
}

impl ResolverExtension for Resolver17 {
    fn new(_subgraph_schemas: Vec<SubgraphSchema>, config: Configuration) -> Result<Self, Error> {
        let config: Config = config.deserialize().unwrap_or(Config::EchoInput { key: None });
        Ok(Self { config })
    }

    fn resolve(
        &mut self,
        ctx: &AuthorizedOperationContext,
        prepared: &[u8],
        _headers: SubgraphHeaders,
        variables: Variables,
    ) -> Result<Response, Error> {
        match &self.config {
            Config::EchoInput { key } => {
                // field which must be resolved. The prepared bytes can be customized to store anything you need in the operation cache.
                let field = ResolvedField::try_from(prepared)?;
                let arguments: Value = field.arguments(&variables)?;
                let dir_args: Value = field.directive().arguments()?;
                Ok(Response::data(json!({
                    "args": arguments,
                    "directive": dir_args,
                    "config": json!({"key": key})
                })))
            }
            Config::EchoContext { authorization_context } => {
                let authorization_context = authorization_context
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
                Ok(Response::data(json!({
                    "hooks_context": String::from_utf8_lossy(&ctx.hooks_context()),
                    "token": ctx.token().as_bytes().map(String::from_utf8_lossy),
                    "authorization_context": authorization_context
                })))
            }
        }
    }

    fn resolve_subscription<'s>(
        &'s mut self,
        ctx: &'s AuthorizedOperationContext,
        prepared: &'s [u8],
        _headers: SubgraphHeaders,
        variables: Variables,
    ) -> Result<impl IntoSubscription<'s>, Error> {
        match &self.config {
            Config::EchoInput { key } => {
                let field = ResolvedField::try_from(prepared)?;
                let arguments: Value = field.arguments(&variables)?;
                let dir_args: Value = field.directive().arguments()?;
                Ok(Subscription {
                    items: vec![
                        serde_json::json!({
                            "args": arguments,
                            "directive": dir_args,
                            "config": json!({"key": key})
                        }),
                        serde_json::json!({
                            "message": "This is a test message from the resolver extension.",
                        }),
                    ]
                    .into(),
                })
            }
            Config::EchoContext { authorization_context } => {
                let authorization_context = authorization_context
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
                Ok(Subscription {
                    items: vec![json!({
                        "hooks_context": String::from_utf8_lossy(&ctx.hooks_context()),
                        "token": ctx.token().as_bytes().map(String::from_utf8_lossy),
                        "authorization_context": authorization_context
                    })]
                    .into(),
                })
            }
        }
    }
}

struct Subscription {
    items: VecDeque<Value>,
}

impl grafbase_sdk::Subscription for Subscription {
    fn next(&mut self) -> Result<Option<SubscriptionItem>, Error> {
        Ok(self.items.pop_front().map(|item| Response::data(item).into()))
    }
}
