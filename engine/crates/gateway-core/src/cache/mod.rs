use std::{future::Future, sync::Arc};

use http::status::StatusCode;

pub use build_key::build_cache_key;
use engine::registry::CachePartialRegistry;
use runtime::cache::{Cache, CacheControl, CacheReadStatus, Cacheable, CachedExecutionResponse};

use super::RequestContext;

mod build_key;
mod key;

#[derive(Clone, Default)]
pub struct CacheConfig {
    pub global_enabled: bool,
    pub subdomain: String,
    pub host_name: String,
    pub cache_control: CacheControl,
    pub partial_registry: CachePartialRegistry,
    pub common_cache_tags: Vec<String>,
}

pub fn process_execution_response<Error, Response>(
    ctx: &impl RequestContext,
    response: Result<CachedExecutionResponse<Arc<engine::Response>>, Error>,
) -> Result<Response, Error>
where
    Error: std::fmt::Display,
    Response: super::Response<Error = Error>,
{
    let (response, headers) = match response {
        Ok(execution_response) => match execution_response {
            CachedExecutionResponse::Cached(cached) => (cached, CacheReadStatus::Hit.into_headers()),
            CachedExecutionResponse::Stale {
                response,
                cache_revalidation: revalidated,
            } => (response, CacheReadStatus::Stale { revalidated }.into_headers()),
            CachedExecutionResponse::Origin { response, cache_read } => (
                response,
                cache_read.map(CacheReadStatus::into_headers).unwrap_or_default(),
            ),
        },
        Err(e) => {
            log::error!(ctx.ray_id(), "Execution error: {}", e);
            return Ok(Response::error(StatusCode::INTERNAL_SERVER_ERROR, "Execution error"));
        }
    };
    Response::engine(response).map(|resp| resp.with_additional_headers(headers))
}

pub async fn cached_execution<Value, Error, ValueFut>(
    cache: Arc<impl Cache<Value = Value> + 'static>,
    cache_key: String,
    config: &CacheConfig,
    ctx: &impl RequestContext,
    execution_fut: ValueFut,
) -> Result<CachedExecutionResponse<Arc<Value>>, Error>
where
    Value: Cacheable + 'static,
    Error: std::fmt::Display + Send,
    ValueFut: Future<Output = Result<Arc<Value>, Error>> + Send + 'static,
{
    runtime::cache::cached_execution(
        cache,
        &runtime::cache::GlobalCacheConfig {
            enabled: config.global_enabled,
            common_cache_tags: config.common_cache_tags.clone(),
            subdomain: config.subdomain.clone(),
        },
        &runtime::cache::RequestCacheConfig {
            enabled: config.partial_registry.enable_caching,
            cache_control: CacheControl {
                no_cache: config.cache_control.no_cache,
                no_store: config.cache_control.no_store,
            },
        },
        cache_key,
        ctx,
        execution_fut,
    )
    .await
}
