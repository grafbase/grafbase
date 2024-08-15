use mini_moka::sync::ConcurrentCacheExt;
use runtime::operation_cache::{OperationCache, OperationCacheFactory};

pub struct InMemoryOperationCacheConfig {
    pub limit: usize,
}

pub struct InMemoryOperationCacheFactory {
    pub config: InMemoryOperationCacheConfig,
}

impl InMemoryOperationCacheFactory {
    pub fn inactive() -> Self {
        InMemoryOperationCacheFactory {
            config: InMemoryOperationCacheConfig { limit: 0 },
        }
    }
}

impl Default for InMemoryOperationCacheFactory {
    fn default() -> Self {
        InMemoryOperationCacheFactory {
            config: InMemoryOperationCacheConfig { limit: 1000 },
        }
    }
}

impl OperationCacheFactory for InMemoryOperationCacheFactory {
    type Cache<V> = InMemoryOperationCache<V>
    where
        V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned;

    async fn create<V>(&self) -> Self::Cache<V>
    where
        V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
    {
        InMemoryOperationCache {
            inner: mini_moka::sync::Cache::builder()
                .max_capacity(self.config.limit as u64)
                .build(),
        }
    }
}

pub struct InMemoryOperationCache<V> {
    inner: mini_moka::sync::Cache<String, V>,
}

impl<V> OperationCache<V> for InMemoryOperationCache<V>
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
{
    async fn insert(&self, key: String, value: V) -> u64 {
        self.inner.insert(key, value);
        self.inner.sync();
        self.inner.entry_count()
    }

    async fn get(&self, key: &String) -> Option<V> {
        self.inner.get(key)
    }
}
