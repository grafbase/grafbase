use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    sync::Arc,
    time::Instant,
};

use futures_util::lock::Mutex;
use runtime::cache::{Cacheable, Entry, EntryState, Result};

pub struct InMemoryCache<T> {
    inner: Mutex<CacheInner<T>>,
}

impl<T> InMemoryCache<T> {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn new_with_time(now: impl Fn() -> Instant + Sync + Send + 'static) -> Self {
        InMemoryCache {
            inner: Mutex::new(CacheInner {
                now: Box::new(now),
                key_to_entry: HashMap::new(),
                deletion_tasks: BinaryHeap::new(),
                tag_to_keys: HashMap::new(),
            }),
        }
    }
}

impl<T> Default for InMemoryCache<T> {
    fn default() -> Self {
        InMemoryCache {
            inner: Mutex::new(CacheInner {
                now: Box::new(Instant::now),
                key_to_entry: HashMap::new(),
                deletion_tasks: BinaryHeap::new(),
                tag_to_keys: HashMap::new(),
            }),
        }
    }
}

struct CacheInner<T> {
    // for testing
    now: Box<dyn Fn() -> Instant + Sync + Send>,
    key_to_entry: HashMap<String, CacheEntry<T>>,
    deletion_tasks: BinaryHeap<DeletionTask>,
    tag_to_keys: HashMap<String, HashSet<String>>,
}

struct CacheEntry<T> {
    state: EntryState,
    value: Arc<T>,
    max_age_at: Instant,
}

#[derive(Debug, PartialEq, Eq)]
struct DeletionTask {
    key: String,
    to_delete_at: Instant,
}

impl PartialOrd for DeletionTask {
    #[allow(clippy::non_canonical_partial_ord_impl)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.to_delete_at
            .partial_cmp(&other.to_delete_at)
            .map(std::cmp::Ordering::reverse)
    }
}

impl Ord for DeletionTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_delete_at.cmp(&other.to_delete_at).reverse()
    }
}

impl<T> CacheInner<T> {
    fn purge(&mut self, now: Instant) {
        let mut deleted = vec![];
        while let Some(DeletionTask { key, to_delete_at }) = self.deletion_tasks.peek() {
            if to_delete_at <= &now {
                self.key_to_entry.remove(key);
                let DeletionTask { key, .. } = self.deletion_tasks.pop().unwrap();
                deleted.push(key);
            } else {
                break;
            }
        }
        for tagged in self.tag_to_keys.values_mut() {
            for key in &deleted {
                tagged.remove(key);
            }
        }
    }
}

#[async_trait::async_trait]
impl<T: Clone + Cacheable + 'static> runtime::cache::Cache for InMemoryCache<T> {
    type Value = T;

    async fn get(&self, key: &str) -> Result<Entry<Self::Value>> {
        let mut inner = self.inner.lock().await;
        let now = (inner.now)();
        inner.purge(now);
        Ok(inner
            .key_to_entry
            .get(key)
            .map(|entry| {
                if now < entry.max_age_at {
                    Entry::Hit(T::clone(entry.value.as_ref()))
                } else {
                    Entry::Stale {
                        response: T::clone(entry.value.as_ref()),
                        state: entry.state,
                        is_early_stale: false,
                    }
                }
            })
            .unwrap_or(Entry::Miss))
    }

    async fn put(&self, key: &str, state: EntryState, value: Arc<Self::Value>, tags: Vec<String>) -> Result<()> {
        let mut inner = self.inner.lock().await;
        let now = (inner.now)();
        inner.purge(now);
        let key = key.to_string();
        inner.key_to_entry.insert(
            key.clone(),
            CacheEntry {
                state,
                value: Arc::clone(&value),
                max_age_at: now.checked_add(value.max_age()).unwrap(),
            },
        );
        for tag in tags {
            inner.tag_to_keys.entry(tag).or_default().insert(key.clone());
        }
        inner.deletion_tasks.push(DeletionTask {
            key,
            to_delete_at: now
                .checked_add(value.max_age() + value.stale_while_revalidate())
                .unwrap(),
        });
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut inner = self.inner.lock().await;
        let now = (inner.now)();
        inner.purge(now);
        inner.key_to_entry.remove(key);
        for tagged in inner.tag_to_keys.values_mut() {
            tagged.remove(key);
        }
        Ok(())
    }

    async fn purge_by_tags(&self, tags: Vec<String>) -> Result<()> {
        let mut inner = self.inner.lock().await;
        let now = (inner.now)();
        inner.purge(now);
        let keys = tags.into_iter().fold(HashSet::new(), |mut acc, tag| {
            acc.extend(inner.tag_to_keys.remove(&tag).unwrap_or_default());
            acc
        });
        for key in keys {
            inner.key_to_entry.remove(&key);
        }
        Ok(())
    }

    // in local there is only one host, the cli itself.
    async fn purge_by_hostname(&self, _hostname: String) -> Result<()> {
        let mut inner = self.inner.lock().await;
        inner.key_to_entry.clear();
        inner.deletion_tasks.clear();
        inner.tag_to_keys.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicU64, Ordering::Relaxed},
            Arc,
        },
        time::{Duration, Instant},
    };

    use runtime::cache::{Cache, Cacheable, Entry, EntryState};

    use super::InMemoryCache;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    struct Dummy {
        value: String,
        max_age: Duration,
        stale_while_revalidate: Duration,
    }

    impl Dummy {
        fn new(value: impl Into<String>, max_age: u64, stale_while_revalidate: u64) -> Self {
            Self {
                value: value.into(),
                max_age: Duration::from_secs(max_age),
                stale_while_revalidate: Duration::from_secs(stale_while_revalidate),
            }
        }
    }

    impl Cacheable for Dummy {
        fn max_age(&self) -> Duration {
            self.max_age
        }

        fn stale_while_revalidate(&self) -> Duration {
            self.stale_while_revalidate
        }

        fn cache_tags(&self) -> Vec<String> {
            vec![]
        }

        fn should_purge_related(&self) -> bool {
            false
        }

        fn should_cache(&self) -> bool {
            false
        }
    }

    #[tokio::test]
    async fn get_put() {
        let offset: &'static AtomicU64 = Box::leak(Box::new(AtomicU64::new(0)));
        let cache = InMemoryCache::<Dummy>::new_with_time({
            let start = Instant::now();
            move || start.checked_add(Duration::from_secs(offset.load(Relaxed))).unwrap()
        });
        let put = |key: &'static str| {
            cache.put(
                key,
                EntryState::Fresh,
                Arc::new(Dummy::new(format!("{key} value"), 10, 20)),
                vec![],
            )
        };

        put("test").await.unwrap();

        assert_eq!(cache.get("unknown").await.unwrap(), Entry::Miss);
        assert_eq!(
            cache.get("test").await.unwrap(),
            Entry::Hit(Dummy::new("test value", 10, 20))
        );
        offset.store(25, Relaxed);
        assert_eq!(
            cache.get("test").await.unwrap(),
            Entry::Stale {
                response: Dummy::new("test value", 10, 20),
                state: EntryState::Fresh,
                is_early_stale: false
            }
        );

        offset.store(31, Relaxed);
        assert_eq!(cache.get("test").await.unwrap(), Entry::Miss);

        put("test").await.unwrap();
        assert_eq!(
            cache.get("test").await.unwrap(),
            Entry::Hit(Dummy::new("test value", 10, 20))
        );
        cache.delete("test").await.unwrap();
        assert_eq!(cache.get("test").await.unwrap(), Entry::Miss);
    }

    #[tokio::test]
    async fn tags() {
        let offset: &'static AtomicU64 = Box::leak(Box::new(AtomicU64::new(0)));
        let cache = InMemoryCache::<Dummy>::new_with_time({
            let start = Instant::now();
            move || start.checked_add(Duration::from_secs(offset.load(Relaxed))).unwrap()
        });
        let put = |key: &'static str, tags: &'static [&'static str]| async {
            cache
                .put(
                    key,
                    EntryState::Fresh,
                    Arc::new(Dummy::new(key.to_string(), 10, 20)),
                    tags.iter().map(ToString::to_string).collect(),
                )
                .await
                .unwrap();
        };
        put("Great Dane", &["large", "dog"]).await;
        put("Saint Bernard", &["large", "dog"]).await;
        put("Basset Hound", &["small", "dog"]).await;
        assert_eq!(
            cache.get("Basset Hound").await.unwrap(),
            Entry::Hit(Dummy::new("Basset Hound", 10, 20))
        );
        assert_eq!(
            cache.get("Great Dane").await.unwrap(),
            Entry::Hit(Dummy::new("Great Dane", 10, 20))
        );
        assert_eq!(
            cache.get("Saint Bernard").await.unwrap(),
            Entry::Hit(Dummy::new("Saint Bernard", 10, 20))
        );

        // multiple keys for a tag;
        cache.purge_by_tags(vec!["large".to_string()]).await.unwrap();
        assert_eq!(
            cache.get("Basset Hound").await.unwrap(),
            Entry::Hit(Dummy::new("Basset Hound", 10, 20))
        );
        assert_eq!(cache.get("Great Dane").await.unwrap(), Entry::Miss);
        assert_eq!(cache.get("Saint Bernard").await.unwrap(), Entry::Miss);

        cache.purge_by_tags(vec!["dog".to_string()]).await.unwrap();
        assert_eq!(cache.get("Basset Hound").await.unwrap(), Entry::Miss);
        assert_eq!(cache.get("Great Dane").await.unwrap(), Entry::Miss);
        assert_eq!(cache.get("Saint Bernard").await.unwrap(), Entry::Miss);
    }
}
