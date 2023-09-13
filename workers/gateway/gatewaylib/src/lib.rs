use std::{collections::HashMap, sync::Arc};

use auth::{AuthContext, AuthError};
use cache::ServerCacheError;
use common_types::auth::ExecutionAuth;
use engine::parser::types::OperationType;
use futures_util::{future::BoxFuture, FutureExt};
use gateway_adapter::{
    CustomerDeploymentConfig, ExecutionEngine, ExecutionError, ExecutionHealthResponse, ExecutionRequest,
    StreamingFormat,
};
use http::StatusCode;
use tracing::{info_span, Instrument};

mod admin;
pub mod auth;
mod cache;

pub use cache::{CacheContext, CacheControl};

#[async_trait::async_trait(?Send)]
pub trait Server {
    type Config: Clone;
    type Context: Context<Config = Self::Config> + CacheContext + AuthContext + 'static;
    type Response: Response<Context = Self::Context>;
    type Cache: runtime_ext::cache::Cache<Value = engine::Response> + 'static;
    type ExecutionEngine: ExecutionEngine<ExecutionResponse = engine::Response, ConfigType = Self::Config> + 'static;

    fn cache(&self) -> &Arc<Self::Cache>;
    fn engine(&self) -> &Arc<Self::ExecutionEngine>;

    async fn authorize_admin_request(
        &self,
        ctx: &Self::Context,
        _request: &async_graphql::Request,
    ) -> Result<(), AdminAuthError>;

    async fn authorize_request(
        &self,
        ctx: &Self::Context,
        _request: &engine::Request,
    ) -> Result<ExecutionAuth, AuthError>;

    async fn health(&self, ctx: &Self::Context) -> Result<Self::Response, ServerError> {
        match Arc::clone(self.engine())
            .health(gateway_adapter::ExecutionHealthRequest {
                config: ctx.gateway_config(),
                execution_headers: ctx.execution_headers(),
            })
            .instrument(info_span!("execution_health"))
            .await
        {
            Ok(resp) => Self::Response::health(ctx, resp),
            Err(e) => Self::Response::error(ctx, &e.to_string(), StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    async fn admin_execute(
        &self,
        ctx: &Arc<Self::Context>,
        request: async_graphql::Request,
    ) -> Result<Self::Response, ServerError> {
        if let Err(err) = self
            .authorize_admin_request(ctx, &request)
            .instrument(info_span!("authorize_admin_request"))
            .await
        {
            return match err {
                AdminAuthError::Unauthorized(msg) => {
                    Self::Response::error(ctx, &format!("Unauthorized {msg}"), http::StatusCode::UNAUTHORIZED)
                }
                AdminAuthError::BadRequest(msg) => Self::Response::error(ctx, &msg, http::StatusCode::BAD_REQUEST),
            };
        }
        Self::Response::admin(
            ctx,
            crate::admin::handle_graphql_request(self.cache(), ctx, request).await,
        )
    }

    // FIXME: ExecutionEngine responds with a worker::Response, so can't avoid it for now.
    async fn execute_stream(
        &self,
        ctx: &Self::Context,
        request: engine::Request,
        streaming_format: StreamingFormat,
    ) -> Result<worker::Response, ServerError> {
        let Ok(auth) = self.authorize_request(ctx, &request)
                           .instrument(info_span!("authorize_request"))
                           .await else {
            let engine_response = engine::Response::from_errors(
                vec![engine::ServerError::new("Unauthorized", None)],
                // doesn't really matter, this is not client facing
                OperationType::Query,
            );

            return worker::Response::from_json(&engine_response.to_graphql_response())
                .map_err(|err| format!("Serialization error: {err}").into());

        };

        match Arc::clone(self.engine())
            .execute_stream(
                ExecutionRequest {
                    request,
                    config: ctx.gateway_config(),
                    auth,
                    closest_aws_region: ctx.closest_aws_region(),
                    execution_headers: ctx.execution_headers(),
                },
                streaming_format,
            )
            .instrument(info_span!("execute_stream"))
            .await
        {
            Ok((response, maybe_future)) => {
                if let Some(future) = maybe_future {
                    Context::wait_until_push(ctx, future);
                }
                Ok(response)
            }
            Err(error) => {
                log::error!(Context::ray_id(ctx), "Execution error: {}", error);
                Ok(worker::Response::error("Execution error", 500)?)
            }
        }
    }

    async fn execute(&self, ctx: &Self::Context, request: engine::Request) -> Result<Self::Response, ServerError> {
        let Ok(auth) = self.authorize_request(ctx, &request)
                           .instrument(info_span!("authorize_request"))
                           .await else {
            return Self::Response::engine(ctx, Arc::new(engine::Response::from_errors(
                vec![engine::ServerError::new("Unauthorized", None)],
                // doesn't really matter, this is not client facing
                OperationType::Query,
            )));
        };

        cache::process_execution_response(
            ctx,
            cache::execute_with_cache(self.cache(), ctx, request, auth, |ctx, request, auth| {
                Arc::clone(self.engine())
                    .execute(ExecutionRequest {
                        request,
                        config: ctx.gateway_config(),
                        auth,
                        closest_aws_region: ctx.closest_aws_region(),
                        execution_headers: ctx.execution_headers(),
                    })
                    .instrument(info_span!("execute"))
                    .map(|res| res.map(Arc::new))
            })
            .await,
        )
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AdminAuthError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

#[derive(thiserror::Error, Debug)]
pub enum ServerError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Execution(#[from] ExecutionError),
    #[error(transparent)]
    Cache(#[from] runtime_ext::cache::Error),
    #[error("Internal Error: {0}")]
    Internal(String),
    #[error("Serialization Error: {0}")]
    Serialization(String),
}

impl From<worker::Error> for ServerError {
    fn from(err: worker::Error) -> Self {
        Self::Internal(err.to_string())
    }
}

impl From<&str> for ServerError {
    fn from(msg: &str) -> Self {
        Self::Internal(msg.to_string())
    }
}

impl From<String> for ServerError {
    fn from(msg: String) -> Self {
        Self::Internal(msg)
    }
}

impl From<ServerCacheError<ExecutionError>> for ServerError {
    fn from(value: ServerCacheError<ExecutionError>) -> Self {
        match value {
            ServerCacheError::Cache(c) => ServerError::Cache(c),
            ServerCacheError::Value(v) => ServerError::Execution(v),
        }
    }
}

pub trait Context {
    type Config;

    fn ray_id(&self) -> &str;
    fn wait_until_push(&self, fut: BoxFuture<'static, ()>);
    fn gateway_config(&self) -> CustomerDeploymentConfig<Self::Config>;
    fn closest_aws_region(&self) -> rusoto_core::Region;
    fn execution_headers(&self) -> HashMap<String, String>;
}

pub trait Response: Sized {
    type Context;

    fn error(ctx: &Self::Context, message: &str, code: http::status::StatusCode) -> Result<Self, ServerError>;
    fn engine(ctx: &Self::Context, response: Arc<engine::Response>) -> Result<Self, ServerError>;
    fn health(ctx: &Self::Context, response: ExecutionHealthResponse) -> Result<Self, ServerError>;
    fn admin(ctx: &Self::Context, response: async_graphql::Response) -> Result<Self, ServerError>;

    fn with_header(self, name: String, value: String) -> Result<Self, ServerError>;
    fn with_headers(mut self, headers: impl IntoIterator<Item = (String, String)>) -> Result<Self, ServerError> {
        for (name, value) in headers {
            self = self.with_header(name, value)?;
        }
        Ok(self)
    }
}
