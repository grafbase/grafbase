mod bridge;
mod cache;
mod entity_cache;
mod fetch;
#[cfg(feature = "wasi")]
mod hooks;
mod hot_cache;
mod kv;
mod log;
mod pg;
pub mod rate_limiting;
#[cfg(feature = "redis")]
pub mod redis;
mod ufd_invoker;

pub use bridge::Bridge;
pub use cache::InMemoryCache;
pub use entity_cache::memory::InMemoryEntityCache;
pub use fetch::NativeFetcher;
pub use hot_cache::{InMemoryHotCache, InMemoryHotCacheFactory};
pub use kv::*;
pub use pg::{LazyPgConnectionsPool, LocalPgTransportFactory};
pub use ufd_invoker::UdfInvokerImpl;

#[cfg(feature = "wasi")]
pub use hooks::{ComponentLoader, HooksWasi, HooksWasiConfig};

pub use crate::log::LogEventReceiverImpl;

pub struct ExecutionContext {
    pub request_id: String,
}
