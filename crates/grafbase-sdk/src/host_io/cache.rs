//! A key-value cache shared with all instances of the extension.

use std::{borrow::Cow, time::Duration};

use crate::wit;

/// The cache is a key-value store shared across Wasm instances. As Wasm is single threaded, the
/// gateway uses a pool of Wasm instances to execute extensions. Cache with the same name will be
/// the same across those instances and share the same data.
pub struct Cache {
    inner: wit::Cache,
    timeout: Duration,
}

/// The builder for the cache. It allows to set the name, size, ttl and timeout of the cache.
pub struct CacheBuilder {
    name: Cow<'static, str>,
    size: usize,
    ttl: Option<Duration>,
    timeout: Duration,
}

impl CacheBuilder {
    /// Time to live for cached entries.
    pub fn time_to_live(mut self, ttl: Option<Duration>) -> Self {
        self.ttl = ttl;
        self
    }

    /// The default timeout to use when retrieving data from the cache.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Builds the cache
    pub fn build(self) -> Cache {
        let Self {
            name,
            size,
            ttl,
            timeout,
        } = self;
        Cache {
            inner: wit::Cache::init(&name, size as u32, ttl.map(|d| d.as_millis() as u64)),
            timeout,
        }
    }
}

impl Cache {
    /// Creates a new cache builder with the given name and size.
    /// Caches are unique for a given name and extension.
    pub fn builder(name: impl Into<Cow<'static, str>>, size: usize) -> CacheBuilder {
        CacheBuilder {
            name: name.into(),
            size,
            ttl: None,
            timeout: Duration::from_secs(5),
        }
    }

    /// Retrieves a value from the cache by key or initialize it with the provided function using
    /// the default timeout. See [get_or_init_with_timeout](Cache::get_or_init_with_timeout) for more details
    pub fn get_or_insert<E>(&self, key: &str, f: impl FnOnce() -> Result<Vec<u8>, E>) -> Result<Vec<u8>, E> {
        self.get_or_insert_with_timeout(key, self.timeout, f)
    }

    /// Retrieves a value from the cache by key or initialize it with the provided function.
    /// If there is no existing value in the cache, the callback function will be immediately
    /// called to fill the cache. All further calls during the callback execution will wait for
    /// the value to be computed. As the callback might crash, a timeout limits how long this
    /// function will wait. Unfortunately it does result in a thundering herd problem where all
    /// Wasm instances will try to compute the value at the same time.
    pub fn get_or_insert_with_timeout<E>(
        &self,
        key: &str,
        timeout: Duration,
        f: impl FnOnce() -> Result<Vec<u8>, E>,
    ) -> Result<Vec<u8>, E> {
        if let Some(value) = self.inner.get_or_reserve(key, timeout.as_millis() as u64) {
            return Ok(value);
        }
        let value = f()?;
        self.inner.insert(key, &value);
        Ok(value)
    }
}
