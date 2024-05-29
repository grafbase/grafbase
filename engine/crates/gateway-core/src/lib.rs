use std::sync::Arc;

use common_types::auth::ExecutionAuth;
use engine::parser::types::OperationType;
use futures_util::FutureExt;
use gateway_v2_auth::AuthService;
use grafbase_tracing::{grafbase_client::Client, metrics::GraphqlOperationMetrics};
pub use runtime::context::RequestContext;
use runtime::{
    auth::AccessToken,
    cache::{Cache, CacheReadStatus, CachedExecutionResponse, X_GRAFBASE_CACHE},
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
    operation_metrics: GraphqlOperationMetrics,
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
        meter: grafbase_tracing::otel::opentelemetry::metrics::Meter,
    ) -> Self {
        Self {
            executor,
            cache,
            cache_config,
            auth,
            authorizer,
            trusted_documents,
            operation_metrics: GraphqlOperationMetrics::build(&meter),
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
        let Some(AccessToken::V1(auth)) = self.auth.authorize(ctx.headers()).await else {
            return Ok((
                Arc::new(engine::Response::from_errors_with_type(
                    vec![engine::ServerError::new("Unauthorized", None)],
                    // doesn't really matter, this is not client facing
                    OperationType::Query,
                )),
                Default::default(),
            ));
        };

        let start = web_time::Instant::now();
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
        let normalized_query = operation_normalizer::normalize(request.query(), request.operation_name()).ok();
        let (response, headers) = self.execute_with_auth(ctx, request, auth).await?;
        if let Some((operation, normalized_query)) = response.graphql_operation.clone().zip(normalized_query) {
            self.operation_metrics.record(
                grafbase_tracing::metrics::GraphqlOperationMetricsAttributes {
                    ty: match operation.r#type {
                        common_types::OperationType::Query { .. } => "query",
                        common_types::OperationType::Mutation => "mutation",
                        common_types::OperationType::Subscription => "subscription",
                    },
                    name: operation.name,
                    normalized_query_hash: blake3::hash(normalized_query.as_bytes()).into(),
                    normalized_query,
                    has_errors: !response.errors.is_empty(),
                    cache_status: headers
                        .get(X_GRAFBASE_CACHE)
                        .and_then(|v| v.to_str().ok().map(|s| s.to_string())),
                    client: Client::extract_from(ctx.headers()),
                },
                start.elapsed(),
            );
        }
        Ok((response, headers))
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
            .authorize(ctx.headers())
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

    async fn execute_with_auth(
        &self,
        ctx: &Arc<Executor::Context>,
        request: engine::Request,
        auth: ExecutionAuth,
    ) -> Result<(Arc<engine::Response>, http::HeaderMap), Executor::Error> {
        if !self.cache_config.global_enabled || !self.cache_config.partial_registry.enable_caching {
            let response = Arc::clone(&self.executor)
                .execute(Arc::clone(ctx), auth, request)
                .await?;

            return Ok((Arc::new(response), Default::default()));
        }

        #[cfg(feature = "partial-caching")]
        if self.cache_config.partial_registry.enable_partial_caching {
            let cache_plan = partial_caching::build_plan(
                request.query(),
                request.operation_name(),
                &self.cache_config.partial_registry,
            );

            match cache_plan {
                Ok(Some(plan)) => {
                    let response = cache::partial::partial_caching_execution(
                        plan,
                        &self.cache,
                        auth,
                        request,
                        &self.executor,
                        ctx,
                    )
                    .await?;

                    return Ok((response, Default::default()));
                }
                Ok(None) => {
                    // None means we should proceed with a normal execution.
                }
                Err(error) => {
                    // This probably indicates a malformed query, but the cache planning doesn't have
                    // especially thorough error reporting in it. So for now I want to pass this to
                    // the actual execution where it'll get a better error message.
                    tracing::warn!("error when building cache plan: {error:?}");
                }
            }
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
}
