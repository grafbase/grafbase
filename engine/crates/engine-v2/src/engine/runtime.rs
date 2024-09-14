use std::future::Future;

use grafbase_telemetry::metrics::EngineMetrics;
use runtime::{entity_cache::EntityCache, kv::KvStore, rate_limiting::RateLimiter};

pub type HooksContext<R> = <<R as Runtime>::Hooks as runtime::hooks::Hooks>::Context;

pub trait Runtime: Send + Sync + 'static {
    type Hooks: runtime::hooks::Hooks;
    type Fetcher: runtime::fetch::Fetcher;
    type OperationCacheFactory: runtime::operation_cache::OperationCacheFactory;

    /// Returns a reference to the fetcher associated with this runtime.
    ///
    /// This fetcher is responsible for fetching data for an operation.
    fn fetcher(&self) -> &Self::Fetcher;

    /// Returns a reference to the key-value store associated with this runtime.
    ///
    /// The key-value store is used for storing and retrieving data in a persistent manner.
    fn kv(&self) -> &KvStore;

    /// Returns a reference to the trusted documents client associated with this runtime.
    ///
    /// The trusted documents client is responsible for managing and accessing trusted documents
    /// within the context of the runtime.
    fn trusted_documents(&self) -> &runtime::trusted_documents_client::Client;

    /// Returns a reference to the engine metrics associated with this runtime.
    ///
    /// The engine metrics provide insights into the performance and behavior of the runtime,
    /// allowing for monitoring and analysis of various aspects of the system.
    fn metrics(&self) -> &EngineMetrics;

    /// Returns a reference to the hooks associated with this runtime.
    ///
    /// The hooks provide a mechanism for customizing the behavior of the runtime
    /// by allowing users to register various callbacks and handlers.
    fn hooks(&self) -> &Self::Hooks;

    /// Returns a reference to the operation cache factory associated with this runtime.
    ///
    /// The operation cache factory is responsible for creating and managing operation caches
    /// that can be used to store and retrieve transient data related to operations within the runtime.
    fn operation_cache_factory(&self) -> &Self::OperationCacheFactory;

    /// Returns a reference to the rate limiter associated with this runtime.
    ///
    /// The rate limiter controls the rate of requests to prevent overloading
    /// resources and ensure fair usage of shared services.
    fn rate_limiter(&self) -> &RateLimiter;

    /// Suspends the execution of the current task for a specified duration.
    ///
    /// This method is asynchronous and will yield control back to the runtime while waiting.
    /// It can be used for implementing delays in operations or polling.
    ///
    /// # Arguments
    ///
    /// * `duration` - The length of time to sleep, specified as a `std::time::Duration`.
    fn sleep(&self, duration: std::time::Duration) -> impl Future<Output = ()> + Send;

    /// Returns a reference to the entity cache associated with this runtime.
    ///
    /// The entity cache is responsible for storing and retrieving subgraph entities.
    fn entity_cache(&self) -> &dyn EntityCache;
}

pub(crate) trait RuntimeExt: Runtime {
    /// Runs a provided asynchronous future with a specified timeout.
    ///
    /// This method will execute the provided future, returning its result if it completes
    /// within the given timeout duration. If the timeout is reached before the future completes,
    /// it will return `None`.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The maximum duration to wait for the future to complete, specified as a `std::time::Duration`.
    /// * `fut` - The asynchronous operation to execute, which must implement the `Future` trait and
    ///           return a value of type `T`.
    ///
    /// # Returns
    ///
    /// * `Some(T)` if the future completes within the timeout, or
    /// * `None` if the timeout elapses before the future completes.
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
