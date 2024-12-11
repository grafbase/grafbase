use std::future::Future;

/// Cache meant to store data in a really fast cache, ideally directly in-memory.
/// It is *not* meant for response caching.
/// It's up to the implementation to decide how to evict values to save space.
///
/// Contract:
/// - values are immutable for a given key
/// - values are serialize-able with postcard
/// - keys are URL-safe strings: ALPHA  DIGIT  "-" / "." / "_" / "~"
///
pub trait OperationCache<V>: Send + Sync + 'static
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
{
    // insert a new cache item, return the current cache size
    fn insert(&self, key: String, value: V) -> impl Future<Output = ()> + Send;
    // moka-cache does require a &String rather than a &str
    #[allow(clippy::ptr_arg)]
    fn get(&self, key: &String) -> impl Future<Output = Option<V>> + Send;
}

impl<V> OperationCache<V> for ()
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
{
    async fn insert(&self, _: String, _: V) {}

    async fn get(&self, _: &String) -> Option<V> {
        None
    }
}
