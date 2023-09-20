use std::{borrow::Cow, collections::HashMap, sync::Arc};

use common_types::auth::ExecutionAuth;
use engine::parser::types::OperationType;
use futures_util::{future::BoxFuture, FutureExt};
use http::status::StatusCode;
use runtime::cache::Cache;
use tracing::{info_span, Instrument};

mod admin;
mod auth;
mod cache;
mod executor;
mod streaming_format;

pub use auth::{authorize_request, AuthError};
pub use cache::{CacheConfig, CacheControl};
pub use executor::Executor;
pub use streaming_format::StreamingFormat;

#[async_trait::async_trait]
pub trait RequestContext: Send + Sync + Sized {
    fn ray_id(&self) -> &str;
    // worker requires a 'static future, so there isn't any choice.
    async fn wait_until(&self, fut: BoxFuture<'static, ()>);

    // Legacy for auth, because it also tries to retrieve it from the query params...
    fn authorization_header(&self) -> Option<String>;
    fn header(&self, name: &str) -> Option<String>;
    fn headers(&self) -> HashMap<String, String>;
}

#[async_trait::async_trait]
pub trait Gateway: Send {
    // A bit tedious but this allows the caller to have an easier time with its error, response and
    // context types.
    type Error: From<Error> + std::error::Error + Send + 'static;
    type Context: RequestContext;
    type Response: Response<Context = Self::Context, Error = Self::Error>;
    type Cache: Cache<Value = engine::Response> + 'static;
    type Executor: Executor<Context = Self::Context, Error = Self::Error, Response = Self::Response> + 'static;

    fn context(&self) -> &Arc<Self::Context>;
    fn cache(&self) -> &Arc<Self::Cache>;
    fn executor(&self) -> &Arc<Self::Executor>;
    fn cache_config(&self) -> Cow<'_, CacheConfig<'_>>;

    async fn authorize_admin_request(&self, _request: &async_graphql::Request) -> Result<(), AdminAuthError>;

    async fn authorize_request(&self, _request: &engine::Request) -> Result<ExecutionAuth, AuthError>;

    async fn admin_execute(&self, request: async_graphql::Request) -> Result<Self::Response, Self::Error> {
        if let Err(err) = self
            .authorize_admin_request(&request)
            .instrument(info_span!("authorize_admin_request"))
            .await
        {
            return Ok(match err {
                AdminAuthError::Unauthorized(msg) => Self::Response::error(
                    self.context(),
                    &format!("Unauthorized {msg}"),
                    http::StatusCode::UNAUTHORIZED,
                )?,
                AdminAuthError::BadRequest(msg) => {
                    Self::Response::error(self.context(), &msg, http::StatusCode::BAD_REQUEST)?
                }
            });
        }
        Self::Response::admin(
            self.context(),
            self::admin::handle_graphql_request(self.context().as_ref(), self.cache(), &self.cache_config(), request)
                .await,
        )
    }

    async fn execute(
        &self,
        request: engine::Request,
        streaming_format: Option<StreamingFormat>,
    ) -> Result<Self::Response, Self::Error> {
        let Ok(auth) = self.authorize_request(&request)
                           .instrument(info_span!("authorize_request"))
                           .await else {
            return Self::Response::engine(self.context(), Arc::new(engine::Response::from_errors(
                vec![engine::ServerError::new("Unauthorized", None)],
                // doesn't really matter, this is not client facing
                OperationType::Query,
            )));
        };

        Ok(if let Some(streaming_format) = streaming_format {
            Arc::clone(self.executor())
                .execute_stream(Arc::clone(self.context()), auth, request, streaming_format)
                .instrument(info_span!("execute_stream"))
                .await?
        } else {
            cache::process_execution_response(
                self.context().as_ref(),
                cache::execute_with_cache(
                    self.cache(),
                    &self.cache_config(),
                    self.context().as_ref(),
                    request,
                    auth,
                    |request, auth| {
                        Arc::clone(self.executor())
                            .execute(Arc::clone(self.context()), auth, request)
                            .instrument(info_span!("execute"))
                            .map(move |res| res.map(Arc::new))
                    },
                )
                .await,
            )?
        })
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
pub enum Error {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Cache(runtime::cache::Error),
    #[error("Serialization Error: {0}")]
    Serialization(String),
}

pub trait Response: Sized + Send {
    type Error;
    type Context;

    fn error(ctx: &Self::Context, message: &str, code: StatusCode) -> Result<Self, Self::Error>;
    fn engine(ctx: &Self::Context, response: Arc<engine::Response>) -> Result<Self, Self::Error>;
    fn admin(ctx: &Self::Context, response: async_graphql::Response) -> Result<Self, Self::Error>;

    fn with_header(self, name: String, value: String) -> Result<Self, Self::Error>;
    fn with_headers(mut self, headers: impl IntoIterator<Item = (String, String)>) -> Result<Self, Self::Error> {
        for (name, value) in headers {
            self = self.with_header(name, value)?;
        }
        Ok(self)
    }
}
