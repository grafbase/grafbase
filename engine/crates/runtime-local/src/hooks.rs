use std::{collections::HashMap, sync::Arc};

use runtime::{
    error::GraphqlError,
    hooks::{EdgeDefinition, HeaderMap, Hooks, NodeDefinition},
};
use tracing::instrument;
pub use wasi_component_loader::{ComponentLoader, Config as HooksConfig};

pub struct HooksWasi(Option<ComponentLoader>);

impl HooksWasi {
    pub fn new(loader: Option<ComponentLoader>) -> Self {
        Self(loader)
    }
}

impl Hooks for HooksWasi {
    type Context = Arc<HashMap<String, String>>;

    #[instrument(skip_all)]
    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), GraphqlError> {
        let Some(ref loader) = self.0 else {
            return Ok((Arc::new(HashMap::new()), headers));
        };
        let context = HashMap::new();

        loader
            .on_gateway_request(context, headers)
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
        _metadata: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
    ) -> Result<(), GraphqlError> {
        let Some(ref loader) = self.0 else {
            return Err(GraphqlError::new(
                "@authorized directive cannot be used, so access was denied",
            ));
        };

        let Ok(arguments) = serde_json::to_string(&arguments) else {
            tracing::error!("authorize_edge_pre_execution error at {definition}: failed to serialize arguemtns");
            return Err(GraphqlError::internal_server_error());
        };
        let mut results = loader
            .authorized(Arc::clone(context), vec![arguments])
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(error) => {
                    tracing::error!("authorize_edge_pre_execution error at {definition}: {error}");
                    GraphqlError::internal_server_error()
                }
                wasi_component_loader::Error::User(error) => error_response_to_user_error(error),
            })?
            .into_iter()
            .map(|result| result.map(error_response_to_user_error));

        match results.next() {
            None => Err(GraphqlError::internal_server_error()),
            Some(None) => Ok(()),
            Some(Some(error)) => Err(error),
        }
    }

    async fn authorize_node_pre_execution<'a>(
        &self,
        _context: &Self::Context,
        _definition: NodeDefinition<'a>,
        _metadata: impl serde::Serialize + serde::de::Deserializer<'a> + Send,
    ) -> Result<(), GraphqlError> {
        todo!()
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
