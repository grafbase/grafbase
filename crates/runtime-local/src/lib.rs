mod entity_cache;
mod fetch;
mod kv;
pub mod operation_cache;
pub mod rate_limiting;
#[cfg(feature = "redis")]
pub mod redis;
#[cfg(feature = "wasi")]
pub mod wasi;

pub use entity_cache::memory::InMemoryEntityCache;
#[cfg(feature = "redis")]
pub use entity_cache::redis::RedisEntityCache;
pub use fetch::NativeFetcher;
pub use kv::*;
pub use operation_cache::InMemoryOperationCache;

pub struct ExecutionContext {
    pub request_id: String,
}
