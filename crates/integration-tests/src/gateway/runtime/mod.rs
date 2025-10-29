mod extension;
mod hooks;

use std::sync::{Arc, Mutex};

use engine::{CachedOperation, Schema};
use extension_catalog::ExtensionCatalog;
use gateway_config::Config;
use grafbase_telemetry::metrics::{self, EngineMetrics};
use runtime::{entity_cache::EntityCache, fetch::dynamic::DynamicFetcher, trusted_documents_client};
use runtime_local::{
    InMemoryEntityCache, InMemoryOperationCache, NativeFetcher,
    rate_limiting::in_memory::key_based::InMemoryRateLimiter,
};
use tokio::sync::watch;

pub use extension::*;
pub use hooks::*;

pub struct InstrumentedOperationCache<V> {
    inner: InMemoryOperationCache<V>,
    keys: Arc<Mutex<Vec<String>>>,
}

impl<V> Default for InstrumentedOperationCache<V>
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
{
    fn default() -> Self {
        Self {
            inner: InMemoryOperationCache::default(),
            keys: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<V> InstrumentedOperationCache<V>
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
{
    pub fn recorded_keys(&self) -> Arc<Mutex<Vec<String>>> {
        Arc::clone(&self.keys)
    }

    pub fn values(&self) -> impl Iterator<Item = V> + '_ {
        self.inner.values()
    }

    pub fn inactive() -> Self {
        Self {
            inner: InMemoryOperationCache::inactive(),
            keys: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<V> runtime::operation_cache::OperationCache<V> for InstrumentedOperationCache<V>
where
    V: Clone + Send + Sync + 'static + serde::Serialize + serde::de::DeserializeOwned,
{
    fn insert(&self, key: String, value: V) -> impl std::future::Future<Output = ()> + Send {
        let keys = Arc::clone(&self.keys);
        let inner = &self.inner;
        async move {
            {
                let mut recorded = keys.lock().unwrap();
                recorded.push(key.clone());
            }
            inner.insert(key, value).await;
        }
    }

    fn get(&self, key: &String) -> impl std::future::Future<Output = Option<V>> + Send {
        self.inner.get(key)
    }
}

pub struct TestRuntime {
    pub fetcher: DynamicFetcher,
    pub trusted_documents: trusted_documents_client::Client,
    pub operation_cache: Arc<InstrumentedOperationCache<Arc<CachedOperation>>>,
    pub metrics: EngineMetrics,
    pub rate_limiter: runtime::rate_limiting::RateLimiter,
    pub entity_cache: InMemoryEntityCache,
    pub engine_extensions: EngineTestExtensions,
    pub gateway_extensions: GatewayTestExtensions,
}

pub(super) struct TestRuntimeBuilder {
    pub trusted_documents: Option<trusted_documents_client::Client>,
    pub fetcher: Option<DynamicFetcher>,
    pub extensions: ExtensionsBuilder,
}

impl TestRuntimeBuilder {
    pub async fn finalize_runtime_and_config(
        self,
        config: &mut Config,
        schema: &Arc<Schema>,
    ) -> anyhow::Result<(TestRuntime, Arc<ExtensionCatalog>)> {
        let TestRuntimeBuilder {
            trusted_documents,
            fetcher,
            extensions,
        } = self;

        let (gateway_extensions, engine_extensions, extension_catalog) =
            extensions.build_and_ingest_catalog_into_config(config, schema).await?;

        let (_, rx) = watch::channel(Default::default());

        let runtime = TestRuntime {
            fetcher: fetcher.unwrap_or_else(|| {
                DynamicFetcher::wrap(NativeFetcher::new(config, schema).expect("couldnt construct NativeFetcher"))
            }),
            trusted_documents: trusted_documents.unwrap_or_else(|| trusted_documents_client::Client::new(())),
            metrics: EngineMetrics::build(&metrics::meter_from_global_provider(), None),
            rate_limiter: InMemoryRateLimiter::runtime_with_watcher(rx),
            entity_cache: InMemoryEntityCache::default(),
            operation_cache: Arc::new(InstrumentedOperationCache::default()),
            engine_extensions,
            gateway_extensions,
        };
        Ok((runtime, extension_catalog))
    }
}

impl TestRuntime {
    pub async fn new(config: &Config, schema: &Schema) -> Self {
        let (_, rx) = watch::channel(Default::default());

        let fetcher =
            DynamicFetcher::wrap(NativeFetcher::new(config, schema).expect("couldnt construct NativeFetcher"));

        Self {
            fetcher,
            trusted_documents: trusted_documents_client::Client::new(()),
            operation_cache: Arc::new(InstrumentedOperationCache::default()),
            metrics: EngineMetrics::build(&metrics::meter_from_global_provider(), None),
            rate_limiter: InMemoryRateLimiter::runtime_with_watcher(rx),
            entity_cache: InMemoryEntityCache::default(),
            engine_extensions: EngineTestExtensions::default(),
            gateway_extensions: GatewayTestExtensions::default(),
        }
    }
}

impl engine::Runtime for TestRuntime {
    type Fetcher = DynamicFetcher;
    type OperationCache = Arc<InstrumentedOperationCache<Arc<CachedOperation>>>;
    type Extensions = EngineTestExtensions;

    fn fetcher(&self) -> &Self::Fetcher {
        &self.fetcher
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
        &self.engine_extensions
    }

    async fn clone_and_adjust_for_contract(&self, schema: &Arc<Schema>) -> Result<Self, String> {
        Ok(TestRuntime {
            fetcher: self.fetcher.clone(),
            trusted_documents: self.trusted_documents.clone(),
            metrics: self.metrics.clone(),
            engine_extensions: self
                .engine_extensions
                .clone_and_adjust_for_contract(schema)
                .await
                .map_err(|err| format!("Failed to adjust extensions for contract: {err}"))?,
            rate_limiter: self.rate_limiter.clone(),
            entity_cache: InMemoryEntityCache::default(),
            operation_cache: Arc::clone(&self.operation_cache),
            gateway_extensions: self.gateway_extensions.clone(),
        })
    }
}
