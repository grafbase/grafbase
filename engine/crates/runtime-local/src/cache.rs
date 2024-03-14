use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    time::Instant,
};

use futures_util::lock::Mutex;
use runtime::cache::{Cache, CacheMetadata, Entry, EntryState, GlobalCacheConfig, Key, Result, StaleEntry};

pub struct InMemoryCache {
    inner: Mutex<CacheInner>,
}

impl InMemoryCache {
    pub fn runtime(config: GlobalCacheConfig) -> Cache {
        Cache::new(Self::default(), config)
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

impl Default for InMemoryCache {
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

struct CacheInner {
    // for testing
    now: Box<dyn Fn() -> Instant + Sync + Send>,
    key_to_entry: HashMap<Key, CacheEntry>,
    deletion_tasks: BinaryHeap<DeletionTask>,
    tag_to_keys: HashMap<String, HashSet<Key>>,
}

#[derive(Debug)]
struct CacheEntry {
    state: EntryState,
    value: Vec<u8>,
    max_age_at: Instant,
    metadata: CacheMetadata,
}

#[derive(Debug, PartialEq, Eq)]
struct DeletionTask {
    key: Key,
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

impl CacheInner {
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
impl runtime::cache::CacheInner for InMemoryCache {
    async fn get(&self, key: &Key) -> Result<Entry<Vec<u8>>> {
        let mut inner = self.inner.lock().await;
        let now = (inner.now)();
        inner.purge(now);
        let res = Ok(inner
            .key_to_entry
            .get(key)
            .map(|entry| {
                if now < entry.max_age_at {
                    Entry::Hit(entry.value.clone())
                } else {
                    Entry::Stale(StaleEntry {
                        value: entry.value.clone(),
                        state: entry.state,
                        is_early_stale: false,
                        metadata: entry.metadata.clone(),
                    })
                }
            })
            .unwrap_or(Entry::Miss));
        res
    }

    async fn put(&self, key: &Key, state: EntryState, value: Vec<u8>, metadata: CacheMetadata) -> Result<()> {
        let mut inner = self.inner.lock().await;
        let now = (inner.now)();
        inner.purge(now);
        inner.key_to_entry.insert(
            key.clone(),
            CacheEntry {
                state,
                value,
                max_age_at: now.checked_add(metadata.max_age).unwrap(),
                metadata: metadata.clone(),
            },
        );
        for tag in metadata.tags {
            inner.tag_to_keys.entry(tag).or_default().insert(key.clone());
        }
        inner.deletion_tasks.push(DeletionTask {
            key: key.clone(),
            to_delete_at: now
                .checked_add(metadata.max_age + metadata.stale_while_revalidate)
                .unwrap(),
        });
        Ok(())
    }

    async fn delete(&self, key: &Key) -> Result<()> {
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
        sync::atomic::{AtomicU64, Ordering::Relaxed},
        time::{Duration, Instant},
    };

    use runtime::cache::{Cache, CacheMetadata, Cacheable, Entry, EntryState, GlobalCacheConfig, StaleEntry};

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
        fn metadata(&self) -> CacheMetadata {
            CacheMetadata {
                max_age: self.max_age,
                stale_while_revalidate: self.stale_while_revalidate,
                tags: vec![],
                should_purge_related: false,
                should_cache: false,
            }
        }
    }

    #[tokio::test]
    async fn get_put() {
        let offset: &'static AtomicU64 = Box::leak(Box::new(AtomicU64::new(0)));
        let cache = Cache::new(
            InMemoryCache::new_with_time({
                let start = Instant::now();
                move || start.checked_add(Duration::from_secs(offset.load(Relaxed))).unwrap()
            }),
            GlobalCacheConfig::default(),
        );
        let dummy = Dummy::new("test value".to_string(), 10, 20);
        let test_key = cache.build_key("test");
        let unknown_key = cache.build_key("unknown");
        cache
            .put_json(&test_key, EntryState::Fresh, &dummy, dummy.metadata())
            .await
            .unwrap();

        assert_eq!(cache.get_json::<Dummy>(&unknown_key).await.unwrap(), Entry::Miss);
        assert_eq!(
            cache.get_json(&test_key).await.unwrap(),
            Entry::Hit(Dummy::new("test value", 10, 20))
        );
        offset.store(25, Relaxed);
        assert_eq!(
            cache.get_json(&test_key).await.unwrap(),
            Entry::Stale(StaleEntry {
                value: dummy.clone(),
                state: EntryState::Fresh,
                is_early_stale: false,
                metadata: dummy.metadata()
            })
        );

        offset.store(31, Relaxed);
        assert_eq!(cache.get_json::<Dummy>(&test_key).await.unwrap(), Entry::Miss);

        cache
            .put_json(&test_key, EntryState::Fresh, &dummy, dummy.metadata())
            .await
            .unwrap();

        cache.delete(&test_key).await.unwrap();
        assert_eq!(cache.get_json::<Dummy>(&test_key).await.unwrap(), Entry::Miss);
    }

    #[tokio::test]
    async fn tags() {
        let offset: &'static AtomicU64 = Box::leak(Box::new(AtomicU64::new(0)));
        let cache = Cache::new(
            InMemoryCache::new_with_time({
                let start = Instant::now();
                move || start.checked_add(Duration::from_secs(offset.load(Relaxed))).unwrap()
            }),
            GlobalCacheConfig::default(),
        );
        let put = |key: &'static str, tags: &'static [&'static str]| async {
            let dummy = Dummy::new(key.to_string(), 10, 20);
            cache
                .put_json(
                    &cache.build_key(key),
                    EntryState::Fresh,
                    &dummy,
                    dummy
                        .metadata()
                        .with_priority_tags(&tags.iter().map(|s| s.to_string()).collect::<Vec<_>>()),
                )
                .await
                .unwrap();
        };
        put("Great Dane", &["large", "dog"]).await;
        put("Saint Bernard", &["large", "dog"]).await;
        put("Basset Hound", &["small", "dog"]).await;
        assert_eq!(
            cache.get_json(&cache.build_key("Basset Hound")).await.unwrap(),
            Entry::Hit(Dummy::new("Basset Hound", 10, 20))
        );
        assert_eq!(
            cache.get_json(&cache.build_key("Great Dane")).await.unwrap(),
            Entry::Hit(Dummy::new("Great Dane", 10, 20))
        );
        assert_eq!(
            cache.get_json(&cache.build_key("Saint Bernard")).await.unwrap(),
            Entry::Hit(Dummy::new("Saint Bernard", 10, 20))
        );

        // multiple keys for a tag;
        cache.purge_by_tags(vec!["large".to_string()]).await.unwrap();
        assert_eq!(
            cache.get_json(&cache.build_key("Basset Hound")).await.unwrap(),
            Entry::Hit(Dummy::new("Basset Hound", 10, 20))
        );
        assert_eq!(
            cache.get_json::<Dummy>(&cache.build_key("Great Dane")).await.unwrap(),
            Entry::Miss
        );
        assert_eq!(
            cache
                .get_json::<Dummy>(&cache.build_key("Saint Bernard"))
                .await
                .unwrap(),
            Entry::Miss
        );

        cache.purge_by_tags(vec!["dog".to_string()]).await.unwrap();
        assert_eq!(
            cache.get_json::<Dummy>(&cache.build_key("Basset Hound")).await.unwrap(),
            Entry::Miss
        );
        assert_eq!(
            cache.get_json::<Dummy>(&cache.build_key("Great Dane")).await.unwrap(),
            Entry::Miss
        );
        assert_eq!(
            cache
                .get_json::<Dummy>(&cache.build_key("Saint Bernard"))
                .await
                .unwrap(),
            Entry::Miss
        );
    }
}
