use grafbase_workspace_hack as _;

use crate::rate_limit::RatelimitContext;
use engine::parser::types::OperationType;
use futures_util::FutureExt;
use grafbase_telemetry::{
    grafbase_client::Client,
    graphql::{GraphqlOperationAttributes, OperationName},
    metrics::{EngineMetrics, GraphqlRequestMetricsAttributes},
    span::graphql::GraphqlOperationSpan,
};
pub use runtime::context::RequestContext;
use runtime::rate_limiting::RateLimiterInner;
use runtime::{
    auth::AccessToken,
    cache::{Cache, CacheReadStatus, CachedExecutionResponse, X_GRAFBASE_CACHE},
};
use std::sync::Arc;
use tracing::{info_span, Instrument};

use engine::{InitialResponse, StreamingPayload};
use futures_util::stream::{self, BoxStream, StreamExt};

mod admin;
mod auth;
mod cache;
mod executor;
mod rate_limit;
mod response;
pub mod serving;
mod streaming;
mod trusted_documents;

pub use crate::cache::build_cache_key;

// Re-exporting these for convenience.
pub use common_types::auth::ExecutionAuth;
pub use gateway_v2_auth::AuthService;

pub use self::{
    auth::{AdminAuthError, Authorizer},
    cache::CacheConfig,
    executor::Executor,
    response::ConstructableResponse,
    streaming::{encode_stream_response, format::StreamingFormat},
};

const CLIENT_NAME_HEADER_NAME: &str = "x-grafbase-client-name";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error(transparent)]
    Cache(runtime::cache::Error),
    #[error(transparent)]
    Ratelimit(#[from] runtime::rate_limiting::Error),
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
    operation_metrics: EngineMetrics,
    rate_limiter: Box<dyn RateLimiterInner>,
}

impl<Executor> Gateway<Executor>
where
    Executor: self::Executor + 'static,
    Executor::Context: RequestContext,
    Executor::Error: std::error::Error + Send + 'static,
    Executor::StreamingResponse: self::ConstructableResponse<Error = Executor::Error>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        executor: Arc<Executor>,
        cache: Cache,
        cache_config: CacheConfig,
        auth: AuthService,
        authorizer: Box<dyn Authorizer<Context = Executor::Context>>,
        trusted_documents: runtime::trusted_documents_client::Client,
        meter: grafbase_telemetry::otel::opentelemetry::metrics::Meter,
        rate_limiter: Box<dyn RateLimiterInner>,
    ) -> Self {
        Self {
            executor,
            cache,
            cache_config,
            auth,
            authorizer,
            trusted_documents,
            operation_metrics: EngineMetrics::build(&meter, None),
            rate_limiter,
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
    ) -> Result<(Arc<engine::Response>, http::HeaderMap), Executor::Error>
    where
        Executor::Error: From<runtime::rate_limiting::Error>,
    {
        let auth = match self
            .auth
            .authenticate(ctx.headers())
            .instrument(info_span!("authorize_request"))
            .await
        {
            Some(auth) if matches!(auth, AccessToken::V1(_)) => auth,
            _ => {
                return Ok((
                    Arc::new(engine::Response::from_errors_with_type(
                        vec![engine::ServerError::new("Unauthorized", None)],
                        // doesn't really matter, this is not client facing
                        OperationType::Query,
                    )),
                    Default::default(),
                ));
            }
        };

        self.rate_limiter
            .limit(&RatelimitContext::new(&request, &auth, ctx.headers()))
            .instrument(info_span!("rate_limit_check"))
            .await?;

        let AccessToken::V1(auth) = auth else {
            unreachable!("auth must be AccessToken::V1 at this point");
        };

        let graphql_span = GraphqlOperationSpan::default();
        let start = web_time::Instant::now();

        async {
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
                    Arc::new(engine::Response::bad_request(vec![err.into()], None)),
                    Default::default(),
                ));
            }

            let normalized_query = operation_normalizer::normalize(request.query(), request.operation_name()).ok();
            let (response, headers) = self.execute_with_auth(ctx, request, auth).await?;
            let status = response.status();
            let elapsed = start.elapsed();

            graphql_span.record_response::<String>(status, &[]);
            if let Some((operation, sanitized_query)) = response.graphql_operation.as_ref().zip(normalized_query) {
                let operation = GraphqlOperationAttributes {
                    ty: match operation.r#type {
                        common_types::OperationType::Query { .. } => grafbase_telemetry::graphql::OperationType::Query,
                        common_types::OperationType::Mutation => grafbase_telemetry::graphql::OperationType::Mutation,
                        common_types::OperationType::Subscription => {
                            grafbase_telemetry::graphql::OperationType::Subscription
                        }
                    },
                    name: operation.name.clone().map(OperationName::Original).unwrap_or_default(),
                    sanitized_query: sanitized_query.into(),
                };
                graphql_span.record_operation(&operation);
                self.operation_metrics.record_operation_duration(
                    GraphqlRequestMetricsAttributes {
                        operation,
                        status,
                        cache_status: headers
                            .get(X_GRAFBASE_CACHE)
                            .and_then(|v| v.to_str().ok().map(|s| s.to_string())),
                        client: Client::extract_from(ctx.headers()),
                    },
                    elapsed,
                );
            }
            Ok((response, headers))
        }
        .instrument(graphql_span.span.clone())
        .await
    }

    pub async fn execute_stream(
        &self,
        ctx: &Arc<Executor::Context>,
        mut request: engine::Request,
        streaming_format: StreamingFormat,
    ) -> Result<Executor::StreamingResponse, Executor::Error>
    where
        Executor::Error: From<runtime::rate_limiting::Error>,
    {
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

        let auth = match self
            .auth
            .authenticate(ctx.headers())
            .instrument(info_span!("authorize_request"))
            .await
        {
            Some(auth) if matches!(auth, AccessToken::V1(_)) => auth,
            _ => {
                return Executor::StreamingResponse::engine(
                    Arc::new(engine::Response::from_errors_with_type(
                        vec![engine::ServerError::new("Unauthorized", None)],
                        // doesn't really matter, this is not client facing
                        OperationType::Query,
                    )),
                    Default::default(),
                );
            }
        };

        self.rate_limiter
            .limit(&RatelimitContext::new(&request, &auth, ctx.headers()))
            .await?;

        let AccessToken::V1(auth) = auth else {
            unreachable!("auth must be AccessToken::V1 at this point");
        };

        Arc::clone(&self.executor)
            .execute_stream(Arc::clone(ctx), auth, request, streaming_format)
            .instrument(info_span!("execute_stream"))
            .await
    }

    pub async fn execute_stream_v2(
        &self,
        ctx: &Arc<Executor::Context>,
        mut request: engine::Request,
    ) -> Result<BoxStream<'static, engine::StreamingPayload>, Executor::Error>
    where
        Executor::Error: From<runtime::rate_limiting::Error>,
    {
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
            return Ok(error_stream(engine::Response {
                errors: vec![err.into()],
                ..Default::default()
            }));
        }

        let auth = match self
            .auth
            .authenticate(ctx.headers())
            .instrument(info_span!("authorize_request"))
            .await
        {
            Some(auth) if matches!(auth, AccessToken::V1(_)) => auth,
            _ => {
                return Ok(error_stream(engine::Response::from_errors_with_type(
                    vec![engine::ServerError::new("Unauthorized", None)],
                    // doesn't really matter, this is not client facing
                    OperationType::Query,
                )));
            }
        };

        self.rate_limiter
            .limit(&RatelimitContext::new(&request, &auth, ctx.headers()))
            .await?;

        let AccessToken::V1(auth) = auth else {
            unreachable!("auth must be AccessToken::V1 at this point");
        };

        let mut cache_plan = None;
        if self.cache_config.partial_registry.enable_partial_caching {
            let planning_result = partial_caching::build_plan(
                request.query(),
                request.operation_name(),
                &self.cache_config.partial_registry,
            );
            match planning_result {
                Ok(Some(plan)) => {
                    cache_plan = Some(plan);
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

        let Some(cache_plan) = cache_plan else {
            return Arc::clone(&self.executor)
                .execute_stream_v2(Arc::clone(ctx), auth, request)
                .instrument(info_span!("execute_stream"))
                .await;
        };

        let stream = cache::partial::partial_caching_stream(
            cache_plan,
            &self.cache,
            auth,
            request,
            &self.executor,
            ctx,
            &self.cache_config.partial_registry,
        )
        .await?;

        Ok(stream)
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
                        &self.cache_config.partial_registry,
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

fn error_stream(response: engine::Response) -> BoxStream<'static, engine::StreamingPayload> {
    stream::once(async move { StreamingPayload::InitialResponse(InitialResponse::error(response)) }).boxed()
}
