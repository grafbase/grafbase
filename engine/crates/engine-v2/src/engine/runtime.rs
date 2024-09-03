use std::future::Future;

use grafbase_telemetry::metrics::EngineMetrics;
use runtime::{entity_cache::EntityCache, kv::KvStore, rate_limiting::RateLimiter};

pub trait Runtime: Send + Sync + 'static {
    type Hooks: runtime::hooks::Hooks;
    type Fetcher: runtime::fetch::Fetcher;
    type OperationCacheFactory: runtime::operation_cache::OperationCacheFactory;

    fn fetcher(&self) -> &Self::Fetcher;
    fn kv(&self) -> &KvStore;
    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client;
    fn metrics(&self) -> &EngineMetrics;
    fn hooks(&self) -> &Self::Hooks;
    fn operation_cache_factory(&self) -> &Self::OperationCacheFactory;
    fn rate_limiter(&self) -> &RateLimiter;
    fn sleep(&self, duration: std::time::Duration) -> impl Future<Output = ()> + Send;
    fn entity_cache(&self) -> &dyn EntityCache;
}

pub(crate) trait RuntimeExt: Runtime {
    async fn with_timeout<T>(&self, timeout: std::time::Duration, fut: impl Future<Output = T> + Send) -> Option<T> {
        use futures_util::{pin_mut, select, FutureExt};

        let timeout = async move {
            self.sleep(timeout).await;
            None
        }
        .fuse();

        let fut = fut.map(|output| Some(output)).fuse();

        pin_mut!(timeout);
        pin_mut!(fut);

        select!(
           output = timeout => output,
           output = fut => output
        )
    }
}

impl<T: Runtime> RuntimeExt for T {}
