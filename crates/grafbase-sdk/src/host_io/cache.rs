//! A key-value cache shared with all instances of the extension.

use std::time::Duration;

/// Retrieves a value from the cache by key, initializing it if not present.
///
/// If the value exists in the cache, deserializes and returns it.
/// If not found, calls the initialization function, caches the result, and returns it.
///
/// # Arguments
///
/// * `key` - The cache key to look up
/// * `init` - Function to initialize the value if not found in cache
///
/// # Errors
///
/// Returns an error if serialization/deserialization fails or if the init function fails
pub fn get<F, T>(key: &str, init: F) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnOnce() -> Result<CachedItem<T>, Box<dyn std::error::Error>>,
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let value = crate::wit::Cache::get(key);

    if let Some(value) = value {
        Ok(minicbor_serde::from_slice(&value)?)
    } else {
        let value = init()?;
        let serialized = minicbor_serde::to_vec(&value.value)?;

        crate::wit::Cache::set(key, &serialized, value.duration.map(|d| d.as_millis() as u64));

        Ok(value.value)
    }
}

/// A value to be stored in the cache with an optional time-to-live duration.
pub struct CachedItem<T> {
    value: T,
    duration: Option<Duration>,
}

impl<T> CachedItem<T>
where
    T: serde::Serialize,
{
    /// Creates a new cached item with the given value and optional TTL duration.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to cache
    /// * `duration` - Optional time-to-live duration after which the item expires
    pub fn new(value: T, duration: Option<Duration>) -> Self
    where
        T: serde::Serialize,
    {
        Self { value, duration }
    }
}
