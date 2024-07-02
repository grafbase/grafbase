mod pool;

use std::{collections::HashMap, sync::Arc};

use deadpool::managed::Pool;
use runtime::{
    error::GraphqlError,
    hooks::{EdgeDefinition, HeaderMap, Hooks, NodeDefinition},
};
use tracing::instrument;
pub use wasi_component_loader::{ComponentLoader, Config as HooksConfig};

use self::pool::{AuthorizationHookManager, GatewayHookManager};

pub struct HooksWasi(Option<HooksWasiInner>);

struct HooksWasiInner {
    gateway_hooks: Pool<GatewayHookManager>,
    authorization_hooks: Pool<AuthorizationHookManager>,
}

impl HooksWasi {
    pub fn new(loader: Option<ComponentLoader>) -> Self {
        match loader.map(Arc::new) {
            Some(loader) => {
                let gateway_mgr = GatewayHookManager::new(loader.clone());
                let authorization_mgr = AuthorizationHookManager::new(loader);

                let gateway_hooks = Pool::builder(gateway_mgr)
                    .build()
                    .expect("only fails if not in a runtime");

                let authorization_hooks = Pool::builder(authorization_mgr)
                    .build()
                    .expect("only fails if not in a runtime");

                Self(Some(HooksWasiInner {
                    gateway_hooks,
                    authorization_hooks,
                }))
            }
            None => Self(None),
        }
    }
}

impl Hooks for HooksWasi {
    type Context = Arc<HashMap<String, String>>;

    #[instrument(skip_all)]
    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), GraphqlError> {
        let Some(ref inner) = self.0 else {
            return Ok((Arc::new(HashMap::new()), headers));
        };

        let mut hook = inner.gateway_hooks.get().await.expect("no io, should not fail");

        hook.call(HashMap::new(), headers)
            .await
            .map(|(ctx, headers)| (Arc::new(ctx), headers))
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(err) => {
                    tracing::error!("on_gateway_request error: {err}");
                    GraphqlError::internal_server_error()
                }
                wasi_component_loader::Error::User(err) => error_response_to_user_error(err),
            })
    }

    #[instrument(skip_all)]
    async fn authorize_edge_pre_execution<'a>(
        &self,
        context: &Self::Context,
        definition: EdgeDefinition<'a>,
        arguments: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
        metadata: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
    ) -> Result<(), GraphqlError> {
        let Some(ref inner) = self.0 else {
            return Err(GraphqlError::new(
                "@authorized directive cannot be used, so access was denied",
            ));
        };

        let Ok(arguments) = serde_json::to_string(&arguments) else {
            tracing::error!("authorize_edge_pre_execution error at {definition}: failed to serialize arguments");
            return Err(GraphqlError::internal_server_error());
        };

        let Ok(metadata) = serde_json::to_string(&metadata) else {
            tracing::error!("authorize_edge_pre_execution error at {definition}: failed to serialize metadata");
            return Err(GraphqlError::internal_server_error());
        };

        let mut instance = inner.authorization_hooks.get().await.expect("no io, should not fail");

        let definition = wasi_component_loader::EdgeDefinition {
            parent_type_name: definition.parent_type_name.to_string(),
            field_name: definition.field_name.to_string(),
        };

        instance
            .authorize_edge_pre_execution(Arc::clone(context), definition, arguments, metadata)
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_edge_pre_execution error at: {error}");
                    GraphqlError::internal_server_error()
                }
                wasi_component_loader::Error::User(error) => error_response_to_user_error(error),
            })?;

        Ok(())
    }

    async fn authorize_node_pre_execution<'a>(
        &self,
        context: &Self::Context,
        definition: NodeDefinition<'a>,
        metadata: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
    ) -> Result<(), GraphqlError> {
        let Some(ref inner) = self.0 else {
            return Err(GraphqlError::new(
                "@authorized directive cannot be used, so access was denied",
            ));
        };

        let Ok(metadata) = serde_json::to_string(&metadata) else {
            tracing::error!("authorize_edge_pre_execution error at {definition}: failed to serialize metadata");
            return Err(GraphqlError::internal_server_error());
        };

        let definition = wasi_component_loader::NodeDefinition {
            type_name: definition.type_name.to_string(),
        };

        let mut instance = inner.authorization_hooks.get().await.expect("no io, should not fail");

        instance
            .authorize_node_pre_execution(Arc::clone(context), definition, metadata)
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_node_pre_execution error at: {error}");
                    GraphqlError::internal_server_error()
                }
                wasi_component_loader::Error::User(error) => error_response_to_user_error(error),
            })?;

        Ok(())
    }
}

fn error_response_to_user_error(error: wasi_component_loader::ErrorResponse) -> GraphqlError {
    let extensions = error
        .extensions
        .into_iter()
        .map(|(key, value)| {
            let value = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));

            (key.into(), value)
        })
        .collect();

    GraphqlError {
        message: error.message.into(),
        extensions,
    }
}
