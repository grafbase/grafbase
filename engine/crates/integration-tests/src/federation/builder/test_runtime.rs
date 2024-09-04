use grafbase_telemetry::metrics::{self, EngineMetrics};
use runtime::{
    entity_cache::EntityCache, fetch::dynamic::DynamicFetcher, hooks::DynamicHooks, trusted_documents_client,
};
use runtime_local::{
    rate_limiting::in_memory::key_based::InMemoryRateLimiter, InMemoryEntityCache, InMemoryKvStore,
    InMemoryOperationCacheFactory, NativeFetcher,
};
use runtime_noop::trusted_documents::NoopTrustedDocuments;
use tokio::sync::watch;

pub struct TestRuntime {
    pub fetcher: DynamicFetcher,
    pub trusted_documents: trusted_documents_client::Client,
    pub kv: runtime::kv::KvStore,
    pub hot_cache_factory: InMemoryOperationCacheFactory,
    pub metrics: EngineMetrics,
    pub hooks: DynamicHooks,
    pub rate_limiter: runtime::rate_limiting::RateLimiter,
    pub entity_cache: InMemoryEntityCache,
}

impl Default for TestRuntime {
    fn default() -> Self {
        let (_, rx) = watch::channel(Default::default());

        Self {
            fetcher: DynamicFetcher::wrap(NativeFetcher::default()),
            trusted_documents: trusted_documents_client::Client::new(NoopTrustedDocuments),
            kv: InMemoryKvStore::runtime(),
            metrics: EngineMetrics::build(&metrics::meter_from_global_provider(), None),
            hooks: Default::default(),
            rate_limiter: InMemoryRateLimiter::runtime_with_watcher(rx),
            entity_cache: InMemoryEntityCache::default(),
            hot_cache_factory: Default::default(),
        }
    }
}

impl engine_v2::Runtime for TestRuntime {
    type Hooks = DynamicHooks;
    type Fetcher = DynamicFetcher;
    type OperationCacheFactory = InMemoryOperationCacheFactory;

    fn fetcher(&self) -> &Self::Fetcher {
        &self.fetcher
    }

    fn kv(&self) -> &runtime::kv::KvStore {
        &self.kv
    }

    fn trusted_documents(&self) -> &trusted_documents_client::Client {
        &self.trusted_documents
    }

    fn hooks(&self) -> &Self::Hooks {
        &self.hooks
    }

    fn operation_cache_factory(&self) -> &Self::OperationCacheFactory {
        &self.hot_cache_factory
    }

    fn rate_limiter(&self) -> &runtime::rate_limiting::RateLimiter {
        &self.rate_limiter
    }

    async fn sleep(&self, duration: std::time::Duration) {
        tokio::time::sleep(duration).await
    }

    fn entity_cache(&self) -> &dyn EntityCache {
        &self.entity_cache
    }

    fn metrics(&self) -> &EngineMetrics {
        &self.metrics
    }
}
