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
#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default, deny_unknown_fields)]
struct Config {
    key: Option<String>,
    authorization_context: Option<Vec<String>>,
}

impl ResolverExtension for Resolver17 {
    fn new(_subgraph_schemas: Vec<SubgraphSchema>, config: Configuration) -> Result<Self, Error> {
        let config: Config = config.deserialize().unwrap_or_default();
        Ok(Self { config })
    }

    fn resolve(
        &mut self,
        ctx: &AuthorizedOperationContext,
        prepared: &[u8],
        headers: SubgraphHeaders,
        variables: Variables,
    ) -> Result<Response, Error> {
        // field which must be resolved. The prepared bytes can be customized to store anything you need in the operation cache.
        let field = ResolvedField::try_from(prepared)?;
        match field.directive().name() {
            "echoInput" => {
                let arguments: Value = field.arguments(&variables)?;
                let dir_args: Value = field.directive().arguments()?;
                Ok(Response::data(json!({
                    "args": arguments,
                    "directive": dir_args,
                    "config": json!({"key": self.config.key})
                })))
            }
            "echoContext" => {
                let authorization_context = self
                    .config
                    .authorization_context
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
            "echo" => {
                let args: EchoArgs = field.directive().arguments()?;
                match args {
                    EchoArgs::Header { header } => {
                        let value = headers
                            .get(header)
                            .and_then(|value| value.to_str().ok().map(ToOwned::to_owned))
                            .unwrap_or_default();
                        Ok(Response::data(value))
                    }
                }
            }
            _ => unimplemented!(),
        }
    }

    fn resolve_subscription<'s>(
        &'s mut self,
        ctx: &'s AuthorizedOperationContext,
        prepared: &'s [u8],
        headers: SubgraphHeaders,
        variables: Variables,
    ) -> Result<impl IntoSubscription<'s>, Error> {
        let field = ResolvedField::try_from(prepared)?;
        match field.directive().name() {
            "echoInput" => {
                let arguments: Value = field.arguments(&variables)?;
                let dir_args: Value = field.directive().arguments()?;
                Ok(Subscription::from(vec![
                    serde_json::json!({
                        "args": arguments,
                        "directive": dir_args,
                        "config": json!({"key": self.config.key})
                    }),
                    serde_json::json!({
                        "message": "This is a test message from the resolver extension.",
                    }),
                ]))
            }
            "echoContext" => {
                let authorization_context = self
                    .config
                    .authorization_context
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
                Ok(Subscription::from(vec![json!({
                    "hooks_context": String::from_utf8_lossy(&ctx.hooks_context()),
                    "token": ctx.token().as_bytes().map(String::from_utf8_lossy),
                    "authorization_context": authorization_context
                })]))
            }
            "echo" => {
                let args: EchoArgs = field.directive().arguments()?;
                match args {
                    EchoArgs::Header { header } => {
                        let value = headers
                            .get(header)
                            .and_then(|value| value.to_str().ok().map(ToOwned::to_owned))
                            .unwrap_or_default();
                        Ok(Subscription::from(vec![json!(value)]))
                    }
                }
            }
            _ => unimplemented!(),
        }
    }
}

struct Subscription {
    items: VecDeque<Value>,
}

impl From<Vec<Value>> for Subscription {
    fn from(items: Vec<Value>) -> Self {
        Self { items: items.into() }
    }
}

impl grafbase_sdk::Subscription for Subscription {
    fn next(&mut self) -> Result<Option<SubscriptionItem>, Error> {
        Ok(self.items.pop_front().map(|item| Response::data(item).into()))
    }
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum EchoArgs {
    Header { header: String },
}
