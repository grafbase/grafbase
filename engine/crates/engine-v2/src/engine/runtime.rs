use futures::future::BoxFuture;
use grafbase_telemetry::otel::opentelemetry::metrics::Meter;
use runtime::{entity_cache::EntityCache, fetch::Fetcher, kv::KvStore, rate_limiting::RateLimiter};

pub trait Runtime: Send + Sync + 'static {
    type Hooks: runtime::hooks::Hooks;
    type OperationCacheFactory: runtime::operation_cache::OperationCacheFactory;

    fn fetcher(&self) -> &Fetcher;
    fn kv(&self) -> &KvStore;
    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client;
    fn meter(&self) -> &Meter;
    fn hooks(&self) -> &Self::Hooks;
    fn operation_cache_factory(&self) -> &Self::OperationCacheFactory;
    fn rate_limiter(&self) -> &RateLimiter;
    fn sleep(&self, duration: std::time::Duration) -> BoxFuture<'static, ()>;
    fn entity_cache(&self) -> &dyn EntityCache;
}
