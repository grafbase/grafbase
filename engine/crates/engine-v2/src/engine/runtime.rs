use grafbase_tracing::otel::opentelemetry::metrics::Meter;
use runtime::{fetch::Fetcher, kv::KvStore};

pub trait Runtime: Send + Sync + 'static {
    type Hooks: runtime::hooks::Hooks;
    type CacheFactory: runtime::hot_cache::HotCacheFactory;

    fn fetcher(&self) -> &Fetcher;
    fn kv(&self) -> &KvStore;
    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client;
    fn meter(&self) -> &Meter;
    fn hooks(&self) -> &Self::Hooks;
    fn cache_factory(&self) -> &Self::CacheFactory;
}
