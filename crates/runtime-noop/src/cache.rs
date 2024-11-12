use runtime::cache::{Cache, CacheInner, CacheMetadata, Entry, EntryState, GlobalCacheConfig, Key, Result};

#[derive(Default)]
pub struct NoopCache;

impl NoopCache {
    pub fn runtime(config: GlobalCacheConfig) -> Cache {
        Cache::new(Self, config)
    }
}

#[async_trait::async_trait]
impl CacheInner for NoopCache {
    async fn get(&self, _key: &Key) -> Result<Entry<Vec<u8>>> {
        Ok(Entry::Miss)
    }

    async fn put(&self, _key: &Key, _state: EntryState, _value: Vec<u8>, _metadata: CacheMetadata) -> Result<()> {
        Ok(())
    }

    async fn delete(&self, _key: &Key) -> Result<()> {
        Ok(())
    }

    async fn purge_by_tags(&self, _tags: Vec<String>) -> Result<()> {
        Ok(())
    }

    async fn purge_by_hostname(&self, _hostname: String) -> Result<()> {
        Ok(())
    }
}
