use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use dashmap::DashMap;
use futures::TryFutureExt;
use tokio::sync::{mpsc, oneshot};
use ulid::Ulid;
type WaitListReceiver = mpsc::Receiver<oneshot::Sender<Arc<[u8]>>>;
type WaitListSender = mpsc::Sender<oneshot::Sender<Arc<[u8]>>>;

#[derive(Clone)]
pub struct Cache(Arc<CacheInner>);

impl std::ops::Deref for Cache {
    type Target = CacheInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct CacheInner {
    cache: mini_moka::sync::Cache<String, Arc<[u8]>>,
    wait_list: DashMap<String, (Ulid, WaitListSender, WaitListReceiver)>,
}

impl Cache {
    pub fn new(max_capacity: usize, ttl: Option<Duration>) -> Self {
        let mut builder = mini_moka::sync::Cache::builder().max_capacity(max_capacity as u64);
        if let Some(ttl) = ttl {
            builder = builder.time_to_live(ttl);
        }
        Self(Arc::new(CacheInner {
            cache: builder.build(),
            wait_list: DashMap::new(),
        }))
    }

    /// Gets a value from the cache by key. If this function returns None, the caller must set a new one.
    pub async fn get(&self, key: &str, timeout: Duration) -> Option<Arc<[u8]>> {
        let key_string = key.to_owned();
        if let Some(value) = self.cache.get(&key_string) {
            return Some(value);
        }

        let Some((wait_list_sender, wait_list_id)) = self.get_or_create_wait_list(key_string).await else {
            // This will short-circuit a `None` out if there is no wait list. The function creates a new list,
            // and the caller must call set with a new value. Subsequent calls will get the wait list
            // and yield until the first caller creates the value.
            return None;
        };

        // We have to have a timeout here, because the guest can do IO to get the cache value in the
        // init. If this never finishes, we will leak memory when new callers are added to the list.
        let fut = tokio::time::timeout(timeout, async move {
            let (value_sender, value_receiver) = oneshot::channel();
            wait_list_sender.send(value_sender).await.ok()?;
            value_receiver.await.ok()
        });

        let fut = fut.inspect_err(|_| {
            tracing::error!("timed out waiting for cached value in extension cache to be available");
        });

        if let Some(value) = fut.await.ok().flatten() {
            return Some(value.clone());
        };

        // This happens only if our wait list timed out. We must clean the list so we do not leak
        // memory.
        if self
            .wait_list
            .remove_if(key, |_, (id, _, _)| *id == wait_list_id)
            .is_some()
        {
            let now = SystemTime::now();

            let timestamp = Duration::from_millis(wait_list_id.timestamp_ms());
            let created_at = SystemTime::UNIX_EPOCH.checked_add(timestamp);

            let time_ago = created_at
                .and_then(|created_at| now.duration_since(created_at).ok())
                .unwrap_or_default()
                .as_secs();

            tracing::info!("Removed dead wait extension list, created {time_ago}s ago");
        }

        None
    }

    /// Sets a value in the cache with an optional time-to-live duration in milliseconds.
    pub async fn insert(&self, key: &str, value: Arc<[u8]>) {
        self.cache.insert(key.to_owned(), value.clone());

        // We remove the wait list so subsequent calls do not add themselves to the list. The value
        // is already in the cache. We use receive all listeners from the wait list, and send the
        // new value for them so they can continue execution.
        if let Some((_, (_, _, mut receiver))) = self.wait_list.remove(key) {
            while let Ok(waiter) = receiver.try_recv() {
                let _ = waiter.send(value.clone()).ok();
            }
        }
    }

    pub fn remove(&self, key: &str) {
        self.wait_list.remove(key);
        self.cache.invalidate(&key.to_owned());
    }

    /// Gets or creates a wait list for the given cache key. The first caller to a cache value that is
    /// missing will create a new wait list, this function returns None and the caller must initialize
    /// a new value in the guest, and set a new value in the cache.
    ///
    /// The subsequent callers for this value will get the wait list, and add themselves to it.
    /// When the first caller sets a new value, this will send the value to everybody waiting
    /// in the wait list.
    async fn get_or_create_wait_list(&self, key: String) -> Option<(WaitListSender, Ulid)> {
        let mut created = false;

        let entry = self
            .wait_list
            .entry(key)
            .and_modify(|(id, sender, receiver)| {
                if sender.is_closed() {
                    let (new_sender, new_receiver) = mpsc::channel::<oneshot::Sender<_>>(1024);

                    *id = Ulid::new();
                    *sender = new_sender;
                    *receiver = new_receiver;
                    created = true;
                }
            })
            .or_insert_with(|| {
                created = true;

                let (sender, receiver) = mpsc::channel(1024);
                (Ulid::new(), sender, receiver)
            });

        if created {
            None
        } else {
            Some((entry.1.clone(), entry.0))
        }
    }
}
