mod entity_cache;
mod fetch;
#[cfg(feature = "wasi")]
pub mod hooks;
mod kv;
mod operation_cache;
pub mod rate_limiting;
#[cfg(feature = "redis")]
pub mod redis;

pub use entity_cache::memory::InMemoryEntityCache;
#[cfg(feature = "redis")]
pub use entity_cache::redis::RedisEntityCache;
pub use fetch::NativeFetcher;
pub use kv::*;
pub use operation_cache::{InMemoryOperationCache, InMemoryOperationCacheFactory};

#[cfg(feature = "wasi")]
pub use hooks::{ComponentLoader, HooksWasi, HooksWasiConfig};

pub struct ExecutionContext {
    pub request_id: String,
}
