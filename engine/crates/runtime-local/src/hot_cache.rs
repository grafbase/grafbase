use runtime::hot_cache::{CachedDataKind, HotCache, HotCacheFactory};

pub struct InMemoryHotCacheFactory;

impl HotCacheFactory for InMemoryHotCacheFactory {
    type Cache<V> = InMemoryHotCache<V>
    where
        V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned;

    async fn create<V>(&self, kind: CachedDataKind) -> Self::Cache<V>
    where
        V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
    {
        // A bit arbitrary for now
        let builder = mini_moka::sync::Cache::builder();
        InMemoryHotCache {
            inner: match kind {
                CachedDataKind::PersistedQuery => builder.max_capacity(100).build(),
                CachedDataKind::Operation => builder.max_capacity(1000).build(),
            },
        }
    }
}

pub struct InMemoryHotCache<V> {
    inner: mini_moka::sync::Cache<String, V>,
}

impl<V> HotCache<V> for InMemoryHotCache<V>
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
{
    async fn insert(&self, key: String, value: V) {
        self.inner.insert(key, value);
    }

    async fn get(&self, key: &String) -> Option<V> {
        self.inner.get(key)
    }
}
