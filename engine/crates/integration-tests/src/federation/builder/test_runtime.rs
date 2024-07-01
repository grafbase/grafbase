use runtime::hooks::DynamicHooks;

pub struct TestRuntime {
    pub fetcher: runtime::fetch::Fetcher,
    pub cache: runtime::cache::Cache,
    pub trusted_documents: runtime::trusted_documents_client::Client,
    pub kv: runtime::kv::KvStore,
    pub meter: grafbase_tracing::otel::opentelemetry::metrics::Meter,
    pub hooks: DynamicHooks,
}

impl Default for TestRuntime {
    fn default() -> Self {
        Self {
            fetcher: runtime_local::NativeFetcher::runtime_fetcher(),
            cache: runtime_local::InMemoryCache::runtime(runtime::cache::GlobalCacheConfig {
                enabled: true,
                ..Default::default()
            }),
            trusted_documents: runtime::trusted_documents_client::Client::new(
                runtime_noop::trusted_documents::NoopTrustedDocuments,
            ),
            kv: runtime_local::InMemoryKvStore::runtime(),
            meter: grafbase_tracing::metrics::meter_from_global_provider(),
            hooks: Default::default(),
        }
    }
}

impl engine_v2::Runtime for TestRuntime {
    type Hooks = DynamicHooks;

    fn fetcher(&self) -> &runtime::fetch::Fetcher {
        &self.fetcher
    }

    fn cache(&self) -> &runtime::cache::Cache {
        &self.cache
    }

    fn kv(&self) -> &runtime::kv::KvStore {
        &self.kv
    }

    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client {
        &self.trusted_documents
    }

    fn meter(&self) -> &grafbase_tracing::otel::opentelemetry::metrics::Meter {
        &self.meter
    }

    fn hooks(&self) -> &Self::Hooks {
        &self.hooks
    }
}
