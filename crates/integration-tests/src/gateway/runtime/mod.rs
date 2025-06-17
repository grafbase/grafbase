mod context;
mod extension;
mod hooks;

use std::sync::Arc;

use engine::{CachedOperation, Schema};
use engine_auth::AuthenticationService;
use extension_catalog::Extension;
use gateway_config::Config;
use grafbase_telemetry::metrics::{self, EngineMetrics};
use runtime::{entity_cache::EntityCache, fetch::dynamic::DynamicFetcher, trusted_documents_client};
use runtime_local::{
    InMemoryEntityCache, InMemoryKvStore, InMemoryOperationCache, NativeFetcher,
    rate_limiting::in_memory::key_based::InMemoryRateLimiter,
};
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
    pub rate_limiter: runtime::rate_limiting::RateLimiter,
    pub entity_cache: InMemoryEntityCache,
    pub extensions: ExtensionsDispatcher,
    pub authentication: engine_auth::AuthenticationService<ExtensionsDispatcher>,
    pub hooks: TestHooks,
}

pub(super) struct TestRuntimeBuilder {
    pub trusted_documents: Option<trusted_documents_client::Client>,
    pub fetcher: Option<DynamicFetcher>,
    pub hooks_extension: Option<Extension>,
    pub extensions: ExtensionsBuilder,
}

impl TestRuntimeBuilder {
    pub async fn finalize_runtime_and_config(
        self,
        config: &mut Config,
        schema: &Arc<Schema>,
    ) -> Result<TestRuntime, String> {
        let TestRuntimeBuilder {
            trusted_documents,
            hooks_extension,
            fetcher,
            extensions,
        } = self;

        let (extensions, catalog) = extensions.build_and_ingest_catalog_into_config(config, schema).await?;

        let kv = InMemoryKvStore::runtime();
        let authentication = engine_auth::AuthenticationService::new(config, &catalog, extensions.clone(), &kv);

        let (_, rx) = watch::channel(Default::default());

        Ok(TestRuntime {
            fetcher: fetcher.unwrap_or_else(|| {
                DynamicFetcher::wrap(NativeFetcher::new(config).expect("couldnt construct NativeFetcher"))
            }),
            trusted_documents: trusted_documents.unwrap_or_else(|| trusted_documents_client::Client::new(())),
            kv,
            metrics: EngineMetrics::build(&metrics::meter_from_global_provider(), None),
            rate_limiter: InMemoryRateLimiter::runtime_with_watcher(rx),
            entity_cache: InMemoryEntityCache::default(),
            operation_cache: InMemoryOperationCache::default(),
            extensions,
            authentication,
            hooks: TestHooks::new(config, hooks_extension).await,
        })
    }
}

impl Default for TestRuntime {
    fn default() -> Self {
        let (_, rx) = watch::channel(Default::default());

        let fetcher =
            DynamicFetcher::wrap(NativeFetcher::new(&Config::default()).expect("couldnt construct NativeFetcher"));

        let kv = InMemoryKvStore::runtime();

        let authentication = engine_auth::AuthenticationService::new(
            &Config::default(),
            &Default::default(),
            ExtensionsDispatcher::default(),
            &kv,
        );

        Self {
            fetcher,
            trusted_documents: trusted_documents_client::Client::new(()),
            kv,
            operation_cache: InMemoryOperationCache::default(),
            metrics: EngineMetrics::build(&metrics::meter_from_global_provider(), None),
            rate_limiter: InMemoryRateLimiter::runtime_with_watcher(rx),
            entity_cache: InMemoryEntityCache::default(),
            extensions: ExtensionsDispatcher::default(),
            hooks: TestHooks::default(),
            authentication,
        }
    }
}

impl engine::Runtime for TestRuntime {
    type Fetcher = DynamicFetcher;
    type OperationCache = InMemoryOperationCache<Arc<CachedOperation>>;
    type Extensions = ExtensionsDispatcher;
    type Authenticate = AuthenticationService<Self::Extensions>;

    fn fetcher(&self) -> &Self::Fetcher {
        &self.fetcher
    }

    fn kv(&self) -> &runtime::kv::KvStore {
        &self.kv
    }

    fn trusted_documents(&self) -> &trusted_documents_client::Client {
        &self.trusted_documents
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

    fn authentication(&self) -> &Self::Authenticate {
        &self.authentication
    }
}
