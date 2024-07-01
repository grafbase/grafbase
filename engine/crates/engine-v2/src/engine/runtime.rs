use grafbase_tracing::otel::opentelemetry::metrics::Meter;
use runtime::{cache::Cache, fetch::Fetcher, kv::KvStore};

pub trait Runtime: Send + Sync + 'static {
    type Hooks: runtime::hooks::Hooks;

    fn fetcher(&self) -> &Fetcher;
    fn cache(&self) -> &Cache;
    fn kv(&self) -> &KvStore;
    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client;
    fn meter(&self) -> &Meter;
    fn hooks(&self) -> &Self::Hooks;
}
