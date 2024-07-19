use runtime::hooks::DynamicHooks;
use runtime_local::InMemoryHotCacheFactory;

pub struct TestRuntime {
    pub fetcher: runtime::fetch::Fetcher,
    pub trusted_documents: runtime::trusted_documents_client::Client,
    pub kv: runtime::kv::KvStore,
    pub meter: grafbase_telemetry::otel::opentelemetry::metrics::Meter,
    pub hooks: DynamicHooks,
    pub rate_limiter: runtime::rate_limiting::RateLimiter,
}

impl Default for TestRuntime {
    fn default() -> Self {
        Self {
            fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
            trusted_documents: runtime::trusted_documents_client::Client::new(
                runtime_noop::trusted_documents::NoopTrustedDocuments,
            ),
            kv: runtime_local::InMemoryKvStore::runtime(),
            meter: grafbase_telemetry::metrics::meter_from_global_provider(),
            hooks: Default::default(),
            rate_limiter: runtime_local::rate_limiting::key_based::InMemoryRateLimiter::runtime(Default::default()),
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

    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client {
        &self.trusted_documents
    }

    fn meter(&self) -> &grafbase_telemetry::otel::opentelemetry::metrics::Meter {
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
