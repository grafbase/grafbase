use std::{
    borrow::Cow,
    collections::{BinaryHeap, HashMap, HashSet},
    sync::Mutex,
    time::{Duration, Instant},
};

use futures_util::future::BoxFuture;
use runtime::{
    async_runtime::AsyncRuntime,
    cache_v2::{Cache, CacheError, Entry, RawCache},
};

pub struct InMemoryCacheV2 {
    inner: Mutex<CacheInner>,
}

impl InMemoryCacheV2 {
    pub fn runtime(async_runtime: AsyncRuntime) -> Cache {
        Cache::new(Self::default(), async_runtime)
    }

    #[cfg(test)]
    pub fn runtime_with_time_offset() -> (Cache, &'static std::sync::atomic::AtomicU64) {
        use std::sync::atomic::AtomicU64;

        use crate::async_runtime::TokioCurrentRuntime;

        let offset: &'static AtomicU64 = Box::leak(Box::new(AtomicU64::new(0)));
        let cache = Cache::new(
            InMemoryCacheV2 {
                inner: Mutex::new(CacheInner {
                    now: Box::new({
                        let start = Instant::now();
                        move || {
                            start
                                .checked_add(Duration::from_secs(offset.load(std::sync::atomic::Ordering::Relaxed)))
                                .unwrap()
                        }
                    }),
                    key_to_entry: HashMap::new(),
                    deletion_tasks: BinaryHeap::new(),
                    tag_to_keys: HashMap::new(),
                }),
            },
            TokioCurrentRuntime::runtime(),
        );
        (cache, offset)
    }
}

impl Default for InMemoryCacheV2 {
    fn default() -> Self {
        InMemoryCacheV2 {
            inner: Mutex::new(CacheInner {
                now: Box::new(Instant::now),
                key_to_entry: HashMap::new(),
                deletion_tasks: BinaryHeap::new(),
                tag_to_keys: HashMap::new(),
            }),
        }
    }
}

impl RawCache for InMemoryCacheV2 {
    fn get<'a>(&'a self, namespace: &'a str, key: &'a str) -> BoxFuture<'a, Result<Entry<Vec<u8>>, CacheError>> {
        Box::pin(async move {
            let key = format!("{}:{}", namespace, key);
            self.inner.lock().unwrap().get(&key)
        })
    }

    fn put<'a>(
        &'a self,
        namespace: &'a str,
        key: &'a str,
        value: Cow<'a, [u8]>,
        tags: Vec<String>,
        max_age: Duration,
        stale_while_revalidate: Duration,
    ) -> BoxFuture<'a, Result<(), CacheError>> {
        Box::pin(async move {
            let key = format!("{}:{}", namespace, key);
            self.inner
                .lock()
                .unwrap()
                .put(&key, value, tags, max_age, stale_while_revalidate)
        })
    }

    fn delete<'a>(&'a self, namespace: &'a str, key: &'a str) -> BoxFuture<'a, Result<(), CacheError>> {
        Box::pin(async move {
            let key = format!("{}:{}", namespace, key);
            self.inner.lock().unwrap().delete(&key)
        })
    }

    fn purge_by_tags(&self, tags: Vec<String>) -> BoxFuture<'_, Result<(), CacheError>> {
        Box::pin(async { self.inner.lock().unwrap().purge_by_tags(tags) })
    }

    fn purge_all(&self) -> BoxFuture<'_, Result<(), CacheError>> {
        Box::pin(async { self.inner.lock().unwrap().purge_all() })
    }
}

struct CacheInner {
    // for testing
    now: Box<dyn Fn() -> Instant + Sync + Send>,
    key_to_entry: HashMap<String, CacheEntry>,
    deletion_tasks: BinaryHeap<DeletionTask>,
    tag_to_keys: HashMap<String, HashSet<String>>,
}

#[derive(Debug)]
struct CacheEntry {
    value: Vec<u8>,
    stale_at: Instant,
    invalid_at: Instant,
}

#[derive(Debug, PartialEq, Eq)]
struct DeletionTask {
    key: String,
    invalid_at: Instant,
}

impl PartialOrd for DeletionTask {
    #[allow(clippy::non_canonical_partial_ord_impl)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.invalid_at
            .partial_cmp(&other.invalid_at)
            .map(std::cmp::Ordering::reverse)
    }
}

impl Ord for DeletionTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.invalid_at.cmp(&other.invalid_at).reverse()
    }
}

impl CacheInner {
    fn purge(&mut self, now: Instant) {
        let mut deleted = vec![];
        while let Some(DeletionTask {
            key,
            invalid_at: to_delete_at,
        }) = self.deletion_tasks.peek()
        {
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

    fn get(&mut self, key: &str) -> Result<Entry<Vec<u8>>, CacheError> {
        let now = (self.now)();
        self.purge(now);
        let res = Ok(self
            .key_to_entry
            .get(key)
            .map(|entry| {
                if now < entry.stale_at {
                    // Entry::Hit(entry.value.clone())
                    Entry::Hit {
                        value: entry.value.clone(),
                        stale_at: entry.stale_at,
                        invalid_at: entry.invalid_at,
                    }
                } else if now < entry.invalid_at {
                    Entry::Stale {
                        value: entry.value.clone(),
                        invalid_at: entry.invalid_at,
                    }
                } else {
                    Entry::Miss
                }
            })
            .unwrap_or(Entry::Miss));
        res
    }

    fn put(
        &mut self,
        key: &str,
        value: Cow<'_, [u8]>,
        tags: Vec<String>,
        max_age: Duration,
        stale_while_revalidate: Duration,
    ) -> Result<(), CacheError> {
        let now = (self.now)();
        self.purge(now);
        let invalid_at = now.checked_add(max_age + stale_while_revalidate).unwrap();
        self.key_to_entry.insert(
            key.to_string(),
            CacheEntry {
                value: value.into_owned(),
                stale_at: now.checked_add(max_age).unwrap(),
                invalid_at,
            },
        );
        for tag in tags {
            self.tag_to_keys.entry(tag).or_default().insert(key.to_string());
        }
        self.deletion_tasks.push(DeletionTask {
            key: key.to_string(),
            invalid_at,
        });
        Ok(())
    }

    fn delete(&mut self, key: &String) -> Result<(), CacheError> {
        let now = (self.now)();
        self.purge(now);
        self.key_to_entry.remove(key);
        for tagged in self.tag_to_keys.values_mut() {
            tagged.remove(key);
        }
        Ok(())
    }

    fn purge_by_tags(&mut self, tags: Vec<String>) -> Result<(), CacheError> {
        let now = (self.now)();
        self.purge(now);
        let keys = tags.into_iter().fold(HashSet::new(), |mut acc, tag| {
            acc.extend(self.tag_to_keys.remove(&tag).unwrap_or_default());
            acc
        });
        for key in keys {
            self.key_to_entry.remove(&key);
        }
        Ok(())
    }

    fn purge_all(&mut self) -> Result<(), CacheError> {
        self.key_to_entry.clear();
        self.deletion_tasks.clear();
        self.tag_to_keys.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::atomic::Ordering::Relaxed, time::Duration};

    use crate::async_runtime::TokioCurrentRuntime;

    use super::InMemoryCacheV2;

    #[tokio::test]
    async fn get_put() {
        let (cache, time_offset) = InMemoryCacheV2::runtime_with_time_offset();
        cache
            .put_json("key", &"value".to_string(), Duration::from_secs(10))
            .with_max_stale(Duration::from_secs(20))
            .await
            .unwrap();

        assert_eq!(cache.get_json::<String>("unknown").await.unwrap(), None);
        assert_eq!(cache.get_json("key").await.unwrap(), Some("value".to_string()));

        time_offset.store(25, Relaxed);
        assert_eq!(cache.get_json("key").await.unwrap(), Some("value".to_string()));

        time_offset.store(31, Relaxed);
        assert_eq!(cache.get_json::<String>("key").await.unwrap(), None);

        cache
            .put_json("key", &"value".to_string(), Duration::from_secs(10))
            .with_max_stale(Duration::from_secs(20))
            .await
            .unwrap();

        cache.delete("key").await.unwrap();
        assert_eq!(cache.get_json::<String>("key").await.unwrap(), None);
    }

    #[tokio::test]
    async fn tags() {
        let cache = InMemoryCacheV2::runtime(TokioCurrentRuntime::runtime());
        let put = |key: &'static str, tags: &'static [&'static str]| async {
            cache
                .put_json(key, &key.to_string(), Duration::from_secs(10))
                .with_tags(tags.iter().map(|s| s.to_string()).collect())
                .with_max_stale(Duration::from_secs(20))
                .await
                .unwrap();
        };
        put("Great Dane", &["large", "dog"]).await;
        put("Saint Bernard", &["large", "dog"]).await;
        put("Basset Hound", &["small", "dog"]).await;
        assert_eq!(
            cache.get_json::<String>("Basset Hound").await.unwrap(),
            Some("Basset Hound".to_string())
        );
        assert_eq!(
            cache.get_json::<String>("Great Dane").await.unwrap(),
            Some("Great Dane".to_string())
        );
        assert_eq!(
            cache.get_json("Saint Bernard").await.unwrap(),
            Some("Saint Bernard".to_string())
        );

        // multiple keys for a tag;
        cache.purge_by_tags(vec!["large".to_string()]).await.unwrap();
        assert_eq!(
            cache.get_json::<String>("Basset Hound").await.unwrap(),
            Some("Basset Hound".to_string())
        );
        assert_eq!(cache.get_json::<String>("Great Dane").await.unwrap(), None);
        assert_eq!(cache.get_json::<String>("Saint Bernard").await.unwrap(), None);

        cache.purge_by_tags(vec!["dog".to_string()]).await.unwrap();
        assert_eq!(cache.get_json::<String>("Basset Hound").await.unwrap(), None);
        assert_eq!(cache.get_json::<String>("Great Dane").await.unwrap(), None);
        assert_eq!(cache.get_json::<String>("Saint Bernard").await.unwrap(), None);
    }
}
