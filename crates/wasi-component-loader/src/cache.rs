use std::{
    future::Future,
    time::{Duration, Instant, SystemTime},
};

use dashmap::DashMap;
use futures::TryFutureExt;
use tokio::sync::{mpsc, oneshot, Semaphore};
use ulid::Ulid;
use wasmtime::{
    component::{LinkerInstance, ResourceType},
    StoreContextMut,
};

use crate::{
    names::{CACHE_GET_FUNCTION, CACHE_RESOURCE, CACHE_SET_FUNCTION},
    state::WasiState,
};

type WaitListReceiver = mpsc::Receiver<oneshot::Sender<Vec<u8>>>;
type WaitListSender = mpsc::Sender<oneshot::Sender<Vec<u8>>>;

pub(crate) fn inject_mapping(types: &mut LinkerInstance<'_, WasiState>) -> crate::Result<()> {
    types.resource(CACHE_RESOURCE, ResourceType::host::<()>(), |_, _| Ok(()))?;
    types.func_wrap_async(CACHE_GET_FUNCTION, cache_get)?;
    types.func_wrap_async(CACHE_SET_FUNCTION, cache_set)?;

    Ok(())
}

type CacheGetResult<'a> = Box<dyn Future<Output = anyhow::Result<(Option<Vec<u8>>,)>> + Send + 'a>;
type CacheSetResult<'a> = Box<dyn Future<Output = anyhow::Result<()>> + Send + 'a>;

fn cache_get(store: StoreContextMut<'_, WasiState>, (key,): (String,)) -> CacheGetResult<'_> {
    Box::new(async move {
        let value = store.data().cache().get(&key).await;

        Ok((value,))
    })
}

fn cache_set(
    store: StoreContextMut<'_, WasiState>,
    (key, value, ttl_ms): (String, Vec<u8>, Option<u64>),
) -> CacheSetResult<'_> {
    Box::new(async move {
        store.data().cache().set(&key, value, ttl_ms).await;
        Ok(())
    })
}

pub(crate) struct Cache {
    cache: DashMap<String, CachedValue>,
    semaphore: Semaphore,
    wait_list: DashMap<String, (Ulid, WaitListSender, WaitListReceiver)>,
}

struct CachedValue {
    data: Vec<u8>,
    expires_at: Option<Instant>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
            semaphore: Semaphore::new(1024),
            wait_list: DashMap::new(),
        }
    }

    /// Gets a value from the cache by key. If this function returns None, the caller must set a new one.
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        if let Some(value) = self.cache.get(key) {
            if value.expires_at.map(|expiry| expiry < Instant::now()) == Some(true) {
                self.cache.remove(key);
            } else {
                return Some(value.data.clone());
            }
        }

        let Some((wait_list_sender, wait_list_id)) = self.get_or_create_wait_list(key).await else {
            // This will short-circuit a `None` out if there is no wait list. The function creates a new list,
            // and the caller must call set with a new value. Subsequent calls will get the wait list
            // and yield until the first caller creates the value.
            return None;
        };

        // We have to have a timeout here, because the guest can do IO to get the cache value in the
        // init. If this never finishes, we will leak memory when new callers are added to the list.
        let fut = tokio::time::timeout(Duration::from_secs(5), async move {
            let (value_sender, value_receiver) = oneshot::channel();
            wait_list_sender.send(value_sender).await.ok()?;
            value_receiver.await.ok()
        });

        let fut = fut.inspect_err(|_| {
            tracing::error!("timed out waiting for cached value in extension cache to be available");
        });

        if let Some(value) = fut.await.ok().flatten() {
            return Some(value);
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
    pub async fn set(&self, key: &str, value: Vec<u8>, ttl_ms: Option<u64>) {
        let cached_value = CachedValue {
            data: value.clone(),
            expires_at: ttl_ms.map(|ms| Instant::now() + std::time::Duration::from_millis(ms)),
        };

        self.cache.insert(key.to_string(), cached_value);

        // We remove the wait list so subsequent calls do not add themselves to the list. The value
        // is already in the cache. We use receive all listeners from the wait list, and send the
        // new value for them so they can continue execution.
        if let Some((_, (_, _, mut receiver))) = self.wait_list.remove(key) {
            while let Ok(waiter) = receiver.try_recv() {
                let _ = waiter.send(value.clone()).ok();
            }
        }
    }

    /// Gets or creates a wait list for the given cache key. The first caller to a cache value that is
    /// missing will create a new wait list, this function returns None and the caller must initialize
    /// a new value in the guest, and set a new value in the cache.
    ///
    /// The subsequent callers for this value will get the wait list, and add themselves to it.
    /// When the first caller sets a new value, this will send the value to everybody waiting
    /// in the wait list.
    async fn get_or_create_wait_list(&self, key: &str) -> Option<(WaitListSender, Ulid)> {
        match self.wait_list.entry(key.to_string()) {
            dashmap::Entry::Occupied(mut entry) => {
                let (id, sender, _) = entry.get();

                if sender.is_closed() {
                    let permit = self.semaphore.acquire().await.expect("we never close the semaphore");
                    let (sender, receiver) = mpsc::channel(1024);
                    let id = Ulid::new();

                    entry.insert((id, sender, receiver));
                    drop(permit);

                    None
                } else {
                    Some((sender.clone(), *id))
                }
            }
            dashmap::Entry::Vacant(entry) => {
                let permit = self.semaphore.acquire().await.expect("we never close the semaphore");
                let (sender, receiver) = mpsc::channel(1024);
                let id = Ulid::new();

                entry.insert((id, sender, receiver));
                drop(permit);

                None
            }
        }
    }
}
