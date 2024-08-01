use grafbase_telemetry::{metrics, otel::opentelemetry};
use runtime::{hooks::DynamicHooks, trusted_documents_client};
use runtime_local::{
    rate_limiting::in_memory::key_based::InMemoryRateLimiter, InMemoryHotCacheFactory, InMemoryKvStore, NativeFetcher,
};
use runtime_noop::trusted_documents::NoopTrustedDocuments;
use tokio::sync::watch;

pub struct TestRuntime {
    pub fetcher: runtime::fetch::Fetcher,
    pub trusted_documents: trusted_documents_client::Client,
    pub kv: runtime::kv::KvStore,
    pub meter: opentelemetry::metrics::Meter,
    pub hooks: DynamicHooks,
    pub rate_limiter: runtime::rate_limiting::RateLimiter,
}

impl Default for TestRuntime {
    fn default() -> Self {
        let (_, rx) = watch::channel(Default::default());

        Self {
            fetcher: NativeFetcher::runtime_fetcher(),
            trusted_documents: trusted_documents_client::Client::new(NoopTrustedDocuments),
            kv: InMemoryKvStore::runtime(),
            meter: metrics::meter_from_global_provider(),
            hooks: Default::default(),
            rate_limiter: InMemoryRateLimiter::runtime_with_watcher(rx),
        }
    }
}

impl engine_v2::Runtime for TestRuntime {
    type Hooks = DynamicHooks;
    type CacheFactory = InMemoryHotCacheFactory;

    fn fetcher(&self) -> &runtime::fetch::Fetcher {
        &self.fetcher
    }

    fn kv(&self) -> &runtime::kv::KvStore {
        &self.kv
    }

    fn trusted_documents(&self) -> &trusted_documents_client::Client {
        &self.trusted_documents
    }

    fn meter(&self) -> &opentelemetry::metrics::Meter {
        &self.meter
    }

    fn hooks(&self) -> &Self::Hooks {
        &self.hooks
    }

    fn cache_factory(&self) -> &Self::CacheFactory {
        &InMemoryHotCacheFactory
    }

    fn rate_limiter(&self) -> &runtime::rate_limiting::RateLimiter {
        &self.rate_limiter
    }

    fn sleep(&self, duration: std::time::Duration) -> futures::prelude::future::BoxFuture<'static, ()> {
        Box::pin(tokio::time::sleep(duration))
    }
}
