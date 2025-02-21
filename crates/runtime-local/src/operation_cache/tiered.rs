use runtime::operation_cache::OperationCache;

use super::{InMemoryOperationCache, redis::RedisOperationCache};

pub struct TieredOperationCache<V> {
    in_memory: InMemoryOperationCache<V>,
    distributed: Option<RedisOperationCache>,
}

impl<V> TieredOperationCache<V> {
    pub fn new(in_memory: InMemoryOperationCache<V>, distributed: Option<RedisOperationCache>) -> Self {
        Self { in_memory, distributed }
    }
}

impl<V> TieredOperationCache<V>
where
    V: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + Clone + 'static,
{
    pub fn values(&self) -> impl Iterator<Item = V> + '_ {
        self.in_memory.values()
    }

    pub fn entry_count(&self) -> usize {
        self.in_memory.entry_count()
    }
}

impl<V> OperationCache<V> for TieredOperationCache<V>
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
{
    async fn insert(&self, key: String, value: V) {
        self.in_memory.insert(key.clone(), value.clone()).await;

        if let Some(distributed) = self.distributed.clone() {
            tokio::spawn(async move {
                distributed.insert(key, value).await;
            });
        }
    }

    async fn get(&self, key: &String) -> Option<V> {
        if let Some(value) = self.in_memory.get(key).await {
            return Some(value);
        };

        let value: V = self.distributed.as_ref()?.get(key).await?;

        self.in_memory.insert(key.to_owned(), value.clone()).await;

        Some(value)
    }
}
