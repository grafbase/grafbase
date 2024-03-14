use futures_util::future::BoxFuture;
use runtime::async_runtime::AsyncRuntime;
use runtime::cache::{
    Cache, CacheInner, CacheMetadata, Entry, EntryState, GlobalCacheConfig, Key, Result as CacheV1Result,
};
use runtime::cache_v2::CacheError as CacheV2Error;

#[derive(Default)]
pub struct NoopCache;

impl NoopCache {
    pub fn runtime(config: GlobalCacheConfig) -> Cache {
        Cache::new(Self, config)
    }

    pub fn runtime_v2(async_runtime: AsyncRuntime) -> runtime::cache_v2::Cache {
        runtime::cache_v2::Cache::new(Self, async_runtime)
    }
}

#[async_trait::async_trait]
impl CacheInner for NoopCache {
    async fn get(&self, _key: &Key) -> CacheV1Result<Entry<Vec<u8>>> {
        Ok(Entry::Miss)
    }

    async fn put(
        &self,
        _key: &Key,
        _state: EntryState,
        _value: Vec<u8>,
        _metadata: CacheMetadata,
    ) -> CacheV1Result<()> {
        Ok(())
    }

    async fn delete(&self, _key: &Key) -> CacheV1Result<()> {
        Ok(())
    }

    async fn purge_by_tags(&self, _tags: Vec<String>) -> CacheV1Result<()> {
        Ok(())
    }

    async fn purge_by_hostname(&self, _hostname: String) -> CacheV1Result<()> {
        Ok(())
    }
}

impl runtime::cache_v2::RawCache for NoopCache {
    fn get<'a>(
        &'a self,
        _namespace: &'a str,
        _key: &'a str,
    ) -> BoxFuture<'a, Result<runtime::cache_v2::Entry<Vec<u8>>, CacheV2Error>> {
        Box::pin(async move { Ok(runtime::cache_v2::Entry::Miss) })
    }

    fn put<'a>(
        &'a self,
        _namespace: &'a str,
        _key: &'a str,
        _value: std::borrow::Cow<'a, [u8]>,
        _tags: Vec<String>,
        _max_age: std::time::Duration,
        _stale_while_revalidate: std::time::Duration,
    ) -> BoxFuture<'a, Result<(), CacheV2Error>> {
        Box::pin(async move { Ok(()) })
    }

    fn delete<'a>(&'a self, _namespace: &'a str, _key: &'a str) -> BoxFuture<'a, Result<(), CacheV2Error>> {
        Box::pin(async move { Ok(()) })
    }

    fn purge_by_tags(&self, _tags: Vec<String>) -> BoxFuture<'_, Result<(), CacheV2Error>> {
        Box::pin(async move { Ok(()) })
    }

    fn purge_all(&self) -> BoxFuture<'_, Result<(), CacheV2Error>> {
        Box::pin(async move { Ok(()) })
    }
}
