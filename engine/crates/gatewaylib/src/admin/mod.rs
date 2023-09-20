use std::{ops::Deref, sync::Arc};

use async_graphql::{EmptySubscription, Schema};
use runtime_ext::cache::Cache;
use send_wrapper::SendWrapper;
use tracing::Instrument;

mod error;
mod graphql;

struct WrappedCache {
    inner: SendWrapper<Arc<dyn Cache<Value = engine::Response> + 'static>>,
}

impl WrappedCache {
    fn new(cache: Arc<impl Cache<Value = engine::Response> + 'static>) -> Self {
        Self {
            inner: SendWrapper::new(cache),
        }
    }
}

impl Deref for WrappedCache {
    type Target = dyn Cache<Value = engine::Response>;

    fn deref(&self) -> &Self::Target {
        &**self.inner
    }
}

struct WrappedContext {
    inner: SendWrapper<Arc<dyn crate::cache::CacheContext + 'static>>,
}

impl WrappedContext {
    fn new(cache: Arc<impl crate::cache::CacheContext + 'static>) -> Self {
        Self {
            inner: SendWrapper::new(cache),
        }
    }
}

impl Deref for WrappedContext {
    type Target = dyn crate::cache::CacheContext;

    fn deref(&self) -> &Self::Target {
        &**self.inner
    }
}

#[tracing::instrument(skip_all)]
pub async fn handle_graphql_request(
    cache: &Arc<impl Cache<Value = engine::Response> + 'static>,
    ctx: &Arc<impl crate::cache::CacheContext + 'static>,
    request: async_graphql::Request,
) -> async_graphql::Response {
    let schema = Schema::build(graphql::Query, graphql::Mutation::default(), EmptySubscription)
        .data(WrappedCache::new(Arc::clone(cache)))
        .data(WrappedContext::new(Arc::clone(ctx)))
        .finish();

    schema
        .execute(request)
        .instrument(tracing::info_span!("admin_request"))
        .await
}
