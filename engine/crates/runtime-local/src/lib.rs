mod bridge;
mod cache;
mod entity_cache;
mod fetch;
#[cfg(feature = "wasi")]
pub mod hooks;
mod kv;
mod log;
mod operation_cache;
mod pg;
pub mod rate_limiting;
#[cfg(feature = "redis")]
pub mod redis;
mod ufd_invoker;

pub use bridge::Bridge;
pub use cache::InMemoryCache;
pub use entity_cache::memory::InMemoryEntityCache;
#[cfg(feature = "redis")]
pub use entity_cache::redis::RedisEntityCache;
pub use fetch::NativeFetcher;
pub use kv::*;
pub use operation_cache::{InMemoryOperationCache, InMemoryOperationCacheFactory};
pub use pg::{LazyPgConnectionsPool, LocalPgTransportFactory};
pub use ufd_invoker::UdfInvokerImpl;

#[cfg(feature = "wasi")]
pub use hooks::{ComponentLoader, HooksWasi, HooksWasiConfig};

pub use crate::log::LogEventReceiverImpl;

pub struct ExecutionContext {
    pub request_id: String,
}
