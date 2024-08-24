use std::future::Future;

pub trait OperationCacheFactory: Send + Sync + 'static {
    type Cache<V>: OperationCache<V>
    where
        V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned;

    /// A new instance provides a convenient interface on how values are handled. Keys
    /// still live in the same namespace and MUST be unique.
    fn create<V>(&self) -> impl Future<Output = Self::Cache<V>> + Send
    where
        V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned;
}

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

// ---------------------------//
// -- No-op implementation -- //
// ---------------------------//
impl OperationCacheFactory for () {
    type Cache<V> = ()
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned;

    async fn create<V>(&self) -> Self::Cache<V>
    where
        V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
    {
    }
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
