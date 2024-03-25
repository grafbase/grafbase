use std::sync::Arc;

use engine::parser::types::OperationType;
use futures_util::FutureExt;
use gateway_v2_auth::AuthService;
pub use runtime::context::RequestContext;
use runtime::{
    auth::AccessToken,
    cache::{Cache, CacheReadStatus, CachedExecutionResponse},
};
use tracing::{info_span, Instrument};

mod admin;
mod auth;
mod cache;
mod executor;
mod response;
pub mod serving;
mod streaming;
mod trusted_documents;

pub use crate::cache::build_cache_key;

pub use auth::{AdminAuthError, Authorizer};
pub use cache::CacheConfig;
pub use executor::Executor;
pub use response::ConstructableResponse;
pub use streaming::{encode_stream_response, format::StreamingFormat};

const CLIENT_NAME_HEADER_NAME: &str = "x-grafbase-client-name";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Cache(runtime::cache::Error),
    #[error("Serialization Error: {0}")]
    Serialization(String),
}

// A bit tedious but this allows the caller to have an easier time with its error, response and
// context types.
pub struct Gateway<Executor: self::Executor> {
    executor: Arc<Executor>,
    cache: Cache,
    cache_config: CacheConfig,
    auth: AuthService,
    trusted_documents: runtime::trusted_documents_client::Client,
    authorizer: Box<dyn Authorizer<Context = Executor::Context>>,
}

impl<Executor> Gateway<Executor>
where
    Executor: self::Executor + 'static,
    Executor::Context: RequestContext,
    Executor::Error: From<Error> + std::error::Error + Send + 'static,
    Executor::StreamingResponse: self::ConstructableResponse<Error = Executor::Error>,
{
    pub fn new(
        executor: Arc<Executor>,
        cache: Cache,
        cache_config: CacheConfig,
        auth: AuthService,
        authorizer: Box<dyn Authorizer<Context = Executor::Context>>,
        trusted_documents: runtime::trusted_documents_client::Client,
    ) -> Self {
        Self {
            executor,
            cache,
            cache_config,
            auth,
            authorizer,
            trusted_documents,
        }
    }

    pub async fn admin_execute(
        &self,
        ctx: &Arc<Executor::Context>,
        request: async_graphql::Request,
    ) -> Result<Executor::StreamingResponse, Executor::Error> {
        if let Err(err) = self
            .authorizer
            .authorize_admin_request(ctx, &request)
            .instrument(info_span!("authorize_admin_request"))
            .await
        {
            return Ok(match err {
                AdminAuthError::Unauthorized(msg) => {
                    Executor::StreamingResponse::error(http::StatusCode::UNAUTHORIZED, &format!("Unauthorized {msg}"))
                }
                AdminAuthError::BadRequest(msg) => {
                    Executor::StreamingResponse::error(http::StatusCode::BAD_REQUEST, &msg)
                }
            });
        }
        Executor::StreamingResponse::admin(
            self::admin::handle_graphql_request(ctx.as_ref(), self.cache.clone(), &self.cache_config, request).await,
        )
    }

    pub async fn execute(
        &self,
        ctx: &Arc<Executor::Context>,
        mut request: engine::Request,
    ) -> Result<(Arc<engine::Response>, http::HeaderMap), Executor::Error> {
        let headers = ctx.headers();
        if let Err(err) = self
            .handle_persisted_query(
                &mut request,
                headers
                    .get(CLIENT_NAME_HEADER_NAME)
                    .and_then(|value| value.to_str().ok()),
                headers,
            )
            .await
        {
            return Ok((
                Arc::new(engine::Response {
                    errors: vec![err.into()],
                    ..Default::default()
                }),
                Default::default(),
            ));
        }

        let Some(AccessToken::V1(auth)) = self
            .auth
            .get_access_token(ctx.headers())
            .instrument(info_span!("authorize_request"))
            .await
        else {
            return Ok((
                Arc::new(engine::Response::from_errors_with_type(
                    vec![engine::ServerError::new("Unauthorized", None)],
                    // doesn't really matter, this is not client facing
                    OperationType::Query,
                )),
                Default::default(),
            ));
        };

        if !self.cache_config.global_enabled || !self.cache_config.partial_registry.enable_caching {
            let response = Arc::clone(&self.executor)
                .execute(Arc::clone(ctx), auth, request)
                .await?;

            return Ok((Arc::new(response), Default::default()));
        }

        match build_cache_key(&self.cache_config, ctx.as_ref(), &request, &auth) {
            Ok(cache_key) => {
                let execution_fut = Arc::clone(&self.executor)
                    .execute(Arc::clone(ctx), auth, request)
                    .instrument(info_span!("execute"))
                    .map(|res| res.map(Arc::new));

                let cached_execution =
                    cache::cached_execution(&self.cache, cache_key, ctx.as_ref(), execution_fut).await;

                cache::process_execution_response(ctx.as_ref(), cached_execution)
            }
            Err(_) => {
                let result = Arc::clone(&self.executor)
                    .execute(Arc::clone(ctx), auth, request)
                    .instrument(info_span!("execute"))
                    .map(|res| res.map(Arc::new))
                    .await?;

                let response = CachedExecutionResponse::Origin {
                    response: result,
                    cache_read: CacheReadStatus::Bypass,
                };

                cache::process_execution_response(ctx.as_ref(), Ok(response))
            }
        }
    }

    pub async fn execute_stream(
        &self,
        ctx: &Arc<Executor::Context>,
        mut request: engine::Request,
        streaming_format: StreamingFormat,
    ) -> Result<Executor::StreamingResponse, Executor::Error> {
        let headers = ctx.headers();
        if let Err(err) = self
            .handle_persisted_query(
                &mut request,
                headers
                    .get(CLIENT_NAME_HEADER_NAME)
                    .and_then(|value| value.to_str().ok()),
                headers,
            )
            .await
        {
            return Executor::StreamingResponse::engine(
                Arc::new(engine::Response {
                    errors: vec![err.into()],
                    ..Default::default()
                }),
                Default::default(),
            );
        }

        let Some(AccessToken::V1(auth)) = self
            .auth
            .get_access_token(ctx.headers())
            .instrument(info_span!("authorize_request"))
            .await
        else {
            return Executor::StreamingResponse::engine(
                Arc::new(engine::Response::from_errors_with_type(
                    vec![engine::ServerError::new("Unauthorized", None)],
                    // doesn't really matter, this is not client facing
                    OperationType::Query,
                )),
                Default::default(),
            );
        };

        Arc::clone(&self.executor)
            .execute_stream(Arc::clone(ctx), auth, request, streaming_format)
            .instrument(info_span!("execute_stream"))
            .await
    }
}
