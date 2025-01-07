use redis::{AsyncCommands, RedisResult};
use runtime::operation_cache::OperationCache;

use crate::redis::Pool;

#[derive(Clone)]
pub struct RedisOperationCache {
    pool: Pool,
    key_prefix: String,
}

impl RedisOperationCache {
    pub fn new(pool: Pool, key_prefix: &str) -> Self {
        RedisOperationCache {
            pool,
            key_prefix: key_prefix.to_string(),
        }
    }
}

impl<V> OperationCache<V> for RedisOperationCache
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
{
    async fn insert(&self, key: String, value: V) {
        let Ok(value) = postcard::to_stdvec(&value) else { return };
        let Ok(mut connection) = self.pool.get().await else {
            return;
        };

        let key = format!("{}{key}", self.key_prefix);

        let result: RedisResult<()> = connection.set(&key, &value).await;
        if let Err(err) = result {
            tracing::warn!("could not decode the data stored in key {key} from redis operation cache: {err}");
        }
    }

    async fn get(&self, key: &String) -> Option<V> {
        let Ok(mut connection) = self.pool.get().await else {
            return None;
        };

        let key = format!("{}{key}", self.key_prefix);

        let result: RedisResult<Option<Vec<u8>>> = connection.get(&key).await;
        let bytes = match result {
            Ok(None) => {
                return None;
            }
            Ok(Some(bytes)) => bytes,
            Err(e) => {
                tracing::warn!("could not fetch the key {key} from redis operation cache: {e}");
                return None;
            }
        };

        match postcard::from_bytes(&bytes) {
            Ok(value) => Some(value),
            Err(err) => {
                tracing::warn!("could not decode the data stored in key {key} from redis operation cache: {err}");
                None
            }
        }
    }
}
