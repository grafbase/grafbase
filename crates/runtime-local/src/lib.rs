mod entity_cache;
pub mod fetch;
pub mod operation_cache;
pub mod rate_limiting;
#[cfg(feature = "redis")]
pub mod redis;

pub use entity_cache::memory::InMemoryEntityCache;
#[cfg(feature = "redis")]
pub use entity_cache::redis::RedisEntityCache;
pub use fetch::NativeFetcher;
pub use operation_cache::InMemoryOperationCache;

pub struct ExecutionContext {
    pub request_id: String,
}
