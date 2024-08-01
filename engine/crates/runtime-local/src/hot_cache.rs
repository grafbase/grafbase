use runtime::hot_cache::{CachedDataKind, HotCache, HotCacheFactory};

pub struct InMemoryHotCacheConfig {
    pub limit: usize,
}

pub struct InMemoryHotCacheFactory {
    pub trusted_documents_config: InMemoryHotCacheConfig,
    pub operation_config: InMemoryHotCacheConfig,
}

impl InMemoryHotCacheFactory {
    pub fn inactive() -> Self {
        InMemoryHotCacheFactory {
            trusted_documents_config: InMemoryHotCacheConfig { limit: 0 },
            operation_config: InMemoryHotCacheConfig { limit: 0 },
        }
    }
}

impl Default for InMemoryHotCacheFactory {
    fn default() -> Self {
        InMemoryHotCacheFactory {
            trusted_documents_config: InMemoryHotCacheConfig { limit: 100 },
            operation_config: InMemoryHotCacheConfig { limit: 1000 },
        }
    }
}

impl HotCacheFactory for InMemoryHotCacheFactory {
    type Cache<V> = InMemoryHotCache<V>
    where
        V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned;

    async fn create<V>(&self, kind: CachedDataKind) -> Self::Cache<V>
    where
        V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
    {
        let config = match kind {
            CachedDataKind::TrustedDocument => &self.trusted_documents_config,
            CachedDataKind::Operation => &self.operation_config,
        };
        InMemoryHotCache {
            inner: mini_moka::sync::Cache::builder()
                .max_capacity(config.limit as u64)
                .build(),
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
