use std::{future::Future, sync::Arc};

use super::RequestContext;
pub use build_key::build_cache_key;
use engine::registry::CachePartialRegistry;
use runtime::cache::{Cache, Cacheable, CachedExecutionResponse};

mod build_key;
mod key;

#[derive(Clone, Default)]
pub struct CacheConfig {
    pub global_enabled: bool,
    pub subdomain: String,
    pub host_name: String,
    pub partial_registry: CachePartialRegistry,
    pub common_cache_tags: Vec<String>,
}

pub fn process_execution_response<Error>(
    _ctx: &impl RequestContext,
    response: Result<CachedExecutionResponse<Arc<engine::Response>>, Error>,
) -> Result<(Arc<engine::Response>, http::HeaderMap), Error>
where
    Error: std::fmt::Display,
{
    Ok(response
        .inspect_err(|error| {
            tracing::error!("Execution error: {}", error);
        })?
        .into_response_and_headers())
}

pub async fn cached_execution<Value, Error, ValueFut>(
    cache: &Cache,
    key: runtime::cache::Key,
    ctx: &impl RequestContext,
    execution_fut: ValueFut,
) -> Result<CachedExecutionResponse<Arc<Value>>, Error>
where
    Value: Cacheable + 'static,
    Error: std::fmt::Display + Send,
    ValueFut: Future<Output = Result<Arc<Value>, Error>> + Send + 'static,
{
    cache.cached_execution(ctx, key, execution_fut).await
}
