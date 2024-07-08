mod pool;

use std::{collections::HashMap, sync::Arc};

use deadpool::managed::Pool;
use runtime::{
    error::{PartialErrorCode, PartialGraphqlError},
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
    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), PartialGraphqlError> {
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
                    PartialGraphqlError::internal_hook_error()
                }
                wasi_component_loader::Error::User(err) => {
                    error_response_to_user_error(err, PartialErrorCode::BadRequest)
                }
            })
    }

    #[instrument(skip_all)]
    async fn authorize_edge_pre_execution<'a>(
        &self,
        context: &Self::Context,
        definition: EdgeDefinition<'a>,
        arguments: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
        metadata: Option<impl serde::Serialize + serde::de::Deserializer<'a> + Send>,
    ) -> Result<(), PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Err(PartialGraphqlError::new(
                "@authorized directive cannot be used, so access was denied",
                PartialErrorCode::Unauthorized,
            ));
        };

        let Ok(arguments) = serde_json::to_string(&arguments) else {
            tracing::error!("authorize_edge_pre_execution error at {definition}: failed to serialize arguments");
            return Err(PartialGraphqlError::internal_server_error());
        };

        let Ok(metadata) = metadata.as_ref().map(serde_json::to_string).transpose() else {
            tracing::error!("authorize_edge_pre_execution error at {definition}: failed to serialize metadata");
            return Err(PartialGraphqlError::internal_server_error());
        };

        let mut instance = inner.authorization_hooks.get().await.expect("no io, should not fail");

        let definition = wasi_component_loader::EdgeDefinition {
            parent_type_name: definition.parent_type_name.to_string(),
            field_name: definition.field_name.to_string(),
        };

        instance
            .authorize_edge_pre_execution(Arc::clone(context), definition, arguments, metadata.unwrap_or_default())
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_edge_pre_execution error at: {error}");
                    PartialGraphqlError::internal_hook_error()
                }
                wasi_component_loader::Error::User(error) => {
                    error_response_to_user_error(error, PartialErrorCode::Unauthorized)
                }
            })?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn authorize_node_pre_execution<'a>(
        &self,
        context: &Self::Context,
        definition: NodeDefinition<'a>,
        metadata: Option<impl serde::Serialize + serde::de::Deserializer<'a> + Send>,
    ) -> Result<(), PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Err(PartialGraphqlError::new(
                "@authorized directive cannot be used, so access was denied",
                PartialErrorCode::Unauthorized,
            ));
        };

        let Ok(metadata) = metadata.as_ref().map(serde_json::to_string).transpose() else {
            tracing::error!("authorize_edge_pre_execution error at {definition}: failed to serialize metadata");
            return Err(PartialGraphqlError::internal_server_error());
        };

        let definition = wasi_component_loader::NodeDefinition {
            type_name: definition.type_name.to_string(),
        };

        let mut instance = inner.authorization_hooks.get().await.expect("no io, should not fail");

        instance
            .authorize_node_pre_execution(Arc::clone(context), definition, metadata.unwrap_or_default())
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_node_pre_execution error at: {error}");
                    PartialGraphqlError::internal_hook_error()
                }
                wasi_component_loader::Error::User(error) => {
                    error_response_to_user_error(error, PartialErrorCode::Unauthorized)
                }
            })?;

        Ok(())
    }

    #[instrument(skip_all)]
    async fn authorize_parent_edge_post_execution<'a>(
        &self,
        context: &Self::Context,
        definition: EdgeDefinition<'a>,
        parents: Vec<String>,
        metadata: Option<impl serde::Serialize + serde::de::Deserializer<'a> + Send>,
    ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Err(PartialGraphqlError::new(
                "@authorized directive cannot be used, so access was denied",
                PartialErrorCode::Unauthorized,
            ));
        };

        let Ok(metadata) = metadata.as_ref().map(serde_json::to_string).transpose() else {
            tracing::error!("authorize_edge_pre_execution error at {definition}: failed to serialize metadata");
            return Err(PartialGraphqlError::internal_server_error());
        };

        let definition = wasi_component_loader::EdgeDefinition {
            parent_type_name: definition.parent_type_name.to_string(),
            field_name: definition.field_name.to_string(),
        };

        let mut instance = inner.authorization_hooks.get().await.expect("no io, should not fail");

        let results = instance
            .authorize_parent_edge_post_execution(
                Arc::clone(context),
                definition,
                parents,
                metadata.unwrap_or_default(),
            )
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_node_pre_execution error at: {error}");
                    PartialGraphqlError::internal_server_error()
                }
                wasi_component_loader::Error::User(error) => {
                    error_response_to_user_error(error, PartialErrorCode::Unauthorized)
                }
            })?
            .into_iter()
            .map(|result| match result {
                Ok(()) => Ok(()),
                Err(error) => Err(error_response_to_user_error(error, PartialErrorCode::Unauthorized)),
            })
            .collect();

        Ok(results)
    }

    #[instrument(skip_all)]
    async fn authorize_edge_node_post_execution<'a>(
        &self,
        context: &Self::Context,
        definition: EdgeDefinition<'a>,
        nodes: Vec<String>,
        metadata: Option<impl serde::Serialize + serde::de::Deserializer<'a> + Send>,
    ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Err(PartialGraphqlError::new(
                "@authorized directive cannot be used, so access was denied",
                PartialErrorCode::Unauthorized,
            ));
        };

        let Ok(metadata) = metadata.as_ref().map(serde_json::to_string).transpose() else {
            tracing::error!("authorize_edge_pre_execution error at {definition}: failed to serialize metadata");
            return Err(PartialGraphqlError::internal_server_error());
        };

        let definition = wasi_component_loader::EdgeDefinition {
            parent_type_name: definition.parent_type_name.to_string(),
            field_name: definition.field_name.to_string(),
        };

        let mut instance = inner.authorization_hooks.get().await.expect("no io, should not fail");

        let result = instance
            .authorize_edge_node_post_execution(Arc::clone(context), definition, nodes, metadata.unwrap_or_default())
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_node_pre_execution error at: {error}");
                    PartialGraphqlError::internal_server_error()
                }
                wasi_component_loader::Error::User(error) => {
                    error_response_to_user_error(error, PartialErrorCode::Unauthorized)
                }
            })?
            .into_iter()
            .map(|result| match result {
                Ok(()) => Ok(()),
                Err(error) => Err(error_response_to_user_error(error, PartialErrorCode::Unauthorized)),
            })
            .collect();

        Ok(result)
    }

    #[instrument(skip_all)]
    async fn authorize_edge_post_execution<'a>(
        &self,
        context: &Self::Context,
        definition: EdgeDefinition<'a>,
        edges: Vec<(String, Vec<String>)>,
        metadata: Option<impl serde::Serialize + serde::de::Deserializer<'a> + Send>,
    ) -> Result<Vec<Result<(), PartialGraphqlError>>, PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Err(PartialGraphqlError::new(
                "@authorized directive cannot be used, so access was denied",
                PartialErrorCode::Unauthorized,
            ));
        };

        let Ok(metadata) = metadata.as_ref().map(serde_json::to_string).transpose() else {
            tracing::error!("authorize_edge_pre_execution error at {definition}: failed to serialize metadata");
            return Err(PartialGraphqlError::internal_server_error());
        };

        let definition = wasi_component_loader::EdgeDefinition {
            parent_type_name: definition.parent_type_name.to_string(),
            field_name: definition.field_name.to_string(),
        };

        let mut instance = inner.authorization_hooks.get().await.expect("no io, should not fail");

        let result = instance
            .authorize_edge_post_execution(Arc::clone(context), definition, edges, metadata.unwrap_or_default())
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_node_pre_execution error at: {error}");
                    PartialGraphqlError::internal_server_error()
                }
                wasi_component_loader::Error::User(error) => {
                    error_response_to_user_error(error, PartialErrorCode::Unauthorized)
                }
            })?
            .into_iter()
            .map(|result| match result {
                Ok(()) => Ok(()),
                Err(error) => Err(error_response_to_user_error(error, PartialErrorCode::Unauthorized)),
            })
            .collect();

        Ok(result)
    }
}

fn error_response_to_user_error(
    error: wasi_component_loader::ErrorResponse,
    code: PartialErrorCode,
) -> PartialGraphqlError {
    let extensions = error
        .extensions
        .into_iter()
        .map(|(key, value)| {
            let value = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));

            (key.into(), value)
        })
        .collect();

    PartialGraphqlError {
        message: error.message.into(),
        code,
        extensions,
    }
}
