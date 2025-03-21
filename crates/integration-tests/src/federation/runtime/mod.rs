mod context;
mod extension;
mod hooks;

use std::sync::Arc;

use engine::CachedOperation;
use gateway_config::Config;
use grafbase_telemetry::metrics::{self, EngineMetrics};
use runtime::{entity_cache::EntityCache, fetch::dynamic::DynamicFetcher, trusted_documents_client};
use runtime_local::{
    InMemoryEntityCache, InMemoryKvStore, InMemoryOperationCache, NativeFetcher,
    rate_limiting::in_memory::key_based::InMemoryRateLimiter,
};
use runtime_noop::trusted_documents::NoopTrustedDocuments;
use tokio::sync::watch;

pub use context::*;
pub use extension::*;
pub use hooks::*;

pub struct TestRuntime {
    pub fetcher: DynamicFetcher,
    pub trusted_documents: trusted_documents_client::Client,
    pub kv: runtime::kv::KvStore,
    pub operation_cache: InMemoryOperationCache<Arc<CachedOperation>>,
    pub metrics: EngineMetrics,
    pub hooks: DynamicHooks,
    pub rate_limiter: runtime::rate_limiting::RateLimiter,
    pub entity_cache: InMemoryEntityCache,
    pub extensions: ExtensionsDispatcher,
}

impl TestRuntime {
    pub fn new(config: &Config) -> Self {
        let (_, rx) = watch::channel(Default::default());

        Self {
            fetcher: DynamicFetcher::wrap(NativeFetcher::new(config).expect("couldnt construct NativeFetcher")),
            trusted_documents: trusted_documents_client::Client::new(NoopTrustedDocuments),
            kv: InMemoryKvStore::runtime(),
            metrics: EngineMetrics::build(&metrics::meter_from_global_provider(), None),
            hooks: Default::default(),
            rate_limiter: InMemoryRateLimiter::runtime_with_watcher(rx),
            entity_cache: InMemoryEntityCache::default(),
            operation_cache: InMemoryOperationCache::default(),
            extensions: Default::default(),
        }
    }
}

impl engine::Runtime for TestRuntime {
    type Hooks = DynamicHooks;
    type Fetcher = DynamicFetcher;
    type OperationCache = InMemoryOperationCache<Arc<CachedOperation>>;
    type Extensions = ExtensionsDispatcher;

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

    fn operation_cache(&self) -> &Self::OperationCache {
        &self.operation_cache
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

    fn extensions(&self) -> &Self::Extensions {
        &self.extensions
    }
}
