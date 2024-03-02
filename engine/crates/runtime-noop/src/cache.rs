use futures_util::future::BoxFuture;
use runtime::{
    async_runtime::AsyncRuntime,
    cache::{Cache, CacheError, RawCache},
};

#[derive(Default)]
pub struct NoopCache;

impl NoopCache {
    pub fn runtime(async_runtime: AsyncRuntime) -> Cache {
        Cache::new(Self, async_runtime)
    }
}

impl RawCache for NoopCache {
    fn get<'a>(
        &'a self,
        _namespace: &'a str,
        _key: &'a str,
    ) -> BoxFuture<'a, Result<runtime::cache::Entry<Vec<u8>>, CacheError>> {
        Box::pin(async move { Ok(runtime::cache::Entry::Miss) })
    }

    fn put<'a>(
        &'a self,
        _namespace: &'a str,
        _key: &'a str,
        _value: std::borrow::Cow<'a, [u8]>,
        _tags: Vec<String>,
        _max_age: std::time::Duration,
        _stale_while_revalidate: std::time::Duration,
    ) -> BoxFuture<'a, Result<(), CacheError>> {
        Box::pin(async move { Ok(()) })
    }

    fn delete<'a>(&'a self, _namespace: &'a str, _key: &'a str) -> BoxFuture<'a, Result<(), CacheError>> {
        Box::pin(async move { Ok(()) })
    }

    fn purge_by_tags(&self, _tags: Vec<String>) -> BoxFuture<'_, Result<(), CacheError>> {
        Box::pin(async move { Ok(()) })
    }

    fn purge_all(&self) -> BoxFuture<'_, Result<(), CacheError>> {
        Box::pin(async move { Ok(()) })
    }
}
