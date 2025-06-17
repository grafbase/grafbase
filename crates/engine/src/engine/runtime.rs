use std::{future::Future, sync::Arc};

use grafbase_telemetry::metrics::EngineMetrics;
use runtime::{entity_cache::EntityCache, extension::ExtensionRuntime, kv::KvStore, rate_limiting::RateLimiter};

use crate::CachedOperation;

pub type ExtensionContext<R> = <<R as Runtime>::Extensions as ExtensionRuntime>::Context;

pub trait Runtime: Send + Sync + 'static {
    type Fetcher: runtime::fetch::Fetcher;
    type OperationCache: runtime::operation_cache::OperationCache<Arc<CachedOperation>>;
    type Extensions: ExtensionRuntime;
    type Authenticate: runtime::authentication::Authenticate<ExtensionContext<Self>>;

    fn fetcher(&self) -> &Self::Fetcher;
    fn kv(&self) -> &KvStore;
    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client;
    fn metrics(&self) -> &EngineMetrics;
    fn operation_cache(&self) -> &Self::OperationCache;
    fn rate_limiter(&self) -> &RateLimiter;
    fn sleep(&self, duration: std::time::Duration) -> impl Future<Output = ()> + Send;
    fn entity_cache(&self) -> &dyn EntityCache;
    fn extensions(&self) -> &Self::Extensions;
    fn authentication(&self) -> &Self::Authenticate;
}

pub(crate) trait RuntimeExt: Runtime {
    async fn with_timeout<T>(&self, timeout: std::time::Duration, fut: impl Future<Output = T> + Send) -> Option<T> {
        use futures_util::{FutureExt, pin_mut, select};

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
