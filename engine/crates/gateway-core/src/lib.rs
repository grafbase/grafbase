use std::sync::Arc;

use engine::{
    parser::types::OperationType, AutomaticallyPersistedQuery, ErrorCode, ErrorExtensionValues,
    PersistedQueryRequestExtension, ServerError,
};
use futures_util::FutureExt;
use runtime::cache::{Cache, CacheReadStatus, CachedExecutionResponse};
pub use runtime::context::RequestContext;
use tracing::{info_span, Instrument};

mod admin;
mod auth;
mod cache;
mod executor;
mod response;
pub mod serving;
mod streaming;

pub use crate::cache::build_cache_key;
pub use auth::{authorize_request, AdminAuthError, AuthError, Authorizer};
pub use cache::CacheConfig;
pub use executor::Executor;
pub use response::ConstructableResponse;
pub use streaming::{encode_stream_response, format::StreamingFormat};

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
        authorizer: Box<dyn Authorizer<Context = Executor::Context>>,
    ) -> Self {
        Self {
            executor,
            cache,
            cache_config,
            authorizer,
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
        streaming_format: Option<StreamingFormat>,
    ) -> Result<Executor::StreamingResponse, Executor::Error> {
        if let Err(err) = self.handle_persisted_query(ctx, &mut request).await {
            let response = engine::Response {
                errors: vec![err.into()],
                ..Default::default()
            };
            return ConstructableResponse::engine(Arc::new(response), Default::default());
        }

        let Ok(auth) = self
            .authorizer
            .authorize_request(ctx, &request)
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

        if let Some(streaming_format) = streaming_format {
            Arc::clone(&self.executor)
                .execute_stream(Arc::clone(ctx), auth, request, streaming_format)
                .instrument(info_span!("execute_stream"))
                .await
        } else {
            if !self.cache_config.global_enabled || !self.cache_config.partial_registry.enable_caching {
                let result = Arc::clone(&self.executor)
                    .execute(Arc::clone(ctx), auth, request)
                    .instrument(info_span!("execute"))
                    .await?;

                return ConstructableResponse::engine(Arc::new(result), Default::default());
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

    async fn handle_persisted_query(
        &self,
        ctx: &Arc<Executor::Context>,
        request: &mut engine::Request,
    ) -> Result<(), PersistedQueryError> {
        let Some(PersistedQueryRequestExtension { version, sha256_hash }) = &request.extensions.persisted_query else {
            return Ok(());
        };

        if *version != 1 {
            return Err(PersistedQueryError::UnsupportedVersion);
        }

        let key = self
            .cache
            .build_key(&format!("apq/sha256_{}", hex::encode(sha256_hash)));
        if !request.query().is_empty() {
            use sha2::{Digest, Sha256};
            let digest = <Sha256 as Digest>::digest(request.query().as_bytes()).to_vec();
            if &digest != sha256_hash {
                return Err(PersistedQueryError::InvalidSha256Hash);
            }
            self.cache
                .put_json(
                    &key,
                    runtime::cache::EntryState::Fresh,
                    &AutomaticallyPersistedQuery::V1 {
                        query: request.query().to_string(),
                    },
                    runtime::cache::CacheMetadata {
                        max_age: std::time::Duration::from_secs(24 * 60 * 60),
                        stale_while_revalidate: std::time::Duration::ZERO,
                        tags: Vec::new(),
                        should_purge_related: false,
                        should_cache: true,
                    },
                )
                .await
                .map_err(|err| {
                    log::error!(ctx.ray_id(), "Cache error: {}", err);
                    PersistedQueryError::InternalServerError
                })?;
            return Ok(());
        }

        match self.cache.get_json::<AutomaticallyPersistedQuery>(&key).await {
            Ok(entry) => {
                if let Some(AutomaticallyPersistedQuery::V1 { query }) = entry.into_value() {
                    request.operation_plan_cache_key.query = query;
                    Ok(())
                } else {
                    Err(PersistedQueryError::NotFound)
                }
            }
            Err(err) => {
                log::error!(ctx.ray_id(), "Cache error: {}", err);
                Err(PersistedQueryError::InternalServerError)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum PersistedQueryError {
    #[error("Persisted query not found")]
    NotFound,
    #[error("Persisted query version not supported")]
    UnsupportedVersion,
    #[error("Invalid persisted query sha256Hash")]
    InvalidSha256Hash,
    #[error("Internal server error")]
    InternalServerError,
}

impl From<PersistedQueryError> for ServerError {
    fn from(err: PersistedQueryError) -> Self {
        let message = err.to_string();
        let error = ServerError::new(message, None);
        if matches!(err, PersistedQueryError::NotFound) {
            ServerError {
                extensions: Some(ErrorExtensionValues(
                    [(
                        "code".to_string(),
                        engine::Value::String(ErrorCode::PersistedQueryNotFound.to_string()),
                    )]
                    .into_iter()
                    .collect(),
                )),
                ..error
            }
        } else {
            error
        }
    }
}
