use runtime::operation_cache::OperationCache;

pub struct InMemoryOperationCache<V> {
    enabled: bool,
    inner: mini_moka::sync::Cache<String, V>,
}

impl<V> Default for InMemoryOperationCache<V>
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
{
    fn default() -> Self {
        Self::new(1000)
    }
}

impl<V> InMemoryOperationCache<V>
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn new(limit: usize) -> Self {
        InMemoryOperationCache {
            enabled: limit != 0,
            inner: mini_moka::sync::Cache::builder().max_capacity(limit as u64).build(),
        }
    }

    pub fn inactive() -> Self {
        Self::new(0)
    }

    pub fn entry_count(&self) -> usize {
        self.inner.entry_count() as usize
    }

    pub fn values(&self) -> impl Iterator<Item = V> + '_ {
        self.inner.iter().map(|item| item.value().clone())
    }
}

impl<V> OperationCache<V> for InMemoryOperationCache<V>
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
{
    async fn insert(&self, key: String, value: V) {
        if self.enabled {
            self.inner.insert(key, value);
        }
    }

    async fn get(&self, key: &String) -> Option<V> {
        if !self.enabled {
            return None;
        }
        self.inner.get(key)
    }
}
