use std::{
    borrow::Cow,
    collections::{BinaryHeap, HashMap, HashSet},
    sync::Mutex,
    time::{Duration, Instant},
};

use futures_util::future::BoxFuture;

struct CacheInner<K, M> {
    // for testing
    now: Box<dyn Fn() -> Instant + Sync + Send>,
    key_to_entry: HashMap<String, CacheEntry<M>>,
    deletion_tasks: BinaryHeap<DeletionTask<K>>,
    tag_to_keys: HashMap<String, HashSet<K>>,
}

#[derive(Debug)]
struct CacheEntry<M> {
    value: Vec<u8>,
    stale_at: Instant,
    invalid_at: Instant,
    metadata: M,
}

#[derive(Debug, PartialEq, Eq)]
struct DeletionTask<K> {
    key: K,
    invalid_at: Instant,
}

impl<K> PartialOrd for DeletionTask<K> {
    #[allow(clippy::non_canonical_partial_ord_impl)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.invalid_at
            .partial_cmp(&other.invalid_at)
            .map(std::cmp::Ordering::reverse)
    }
}

impl<K> Ord for DeletionTask<K> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.invalid_at.cmp(&other.invalid_at).reverse()
    }
}

impl<K, M> CacheInner<K, M>
where
    K: std::hash::Hash + Eq,
{
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

    fn get(&mut self, key: &str) -> Option<Entry<Vec<u8>>> {
        let now = (self.now)();
        self.purge(now);
        Some(
            self.key_to_entry
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
                .unwrap_or(Entry::Miss),
        )
    }

    fn put(
        &mut self,
        key: &str,
        value: Cow<'_, [u8]>,
        tags: Vec<String>,
        max_age: Duration,
        stale_while_revalidate: Duration,
        metadata: M,
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
                metadata,
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
