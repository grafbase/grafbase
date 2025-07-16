use std::sync::Arc;

use super::cache::Cache;
use dashmap::DashMap;
use extension_catalog::ExtensionId;
use grafbase_telemetry::{metrics::meter_from_global_provider, otel::opentelemetry::metrics::Histogram};
use sqlx::Postgres;
use wasmtime::component::Resource;
use wasmtime_wasi::{
    ResourceTable,
    p2::{IoView, WasiCtx, WasiView},
};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

use crate::{
    extension::ExtensionConfig,
    resources::{self, FileLogger, GrpcClient},
};

/// Represents the state of the WASI environment.
///
/// This structure encapsulates the WASI context, HTTP context, and a resource table
/// for managing shared resources in memory. It provides methods to create new instances,
/// manage resources, and access the contexts.
pub(crate) struct WasiState {
    /// The WASI context that contains the state for the WASI environment.
    ctx: WasiCtx,

    /// The WASI HTTP context that handles HTTP-related operations.
    http_ctx: WasiHttpCtx,

    /// The resource table that manages shared resources in memory.
    pub table: ResourceTable,

    /// The histogram for request durations.
    request_durations: Histogram<u64>,

    /// A client for making HTTP requests from the guest.
    http_client: reqwest::Client,

    /// A cache to be used for storing data between calls to different instances of the same extension.
    cache: Arc<Cache>,

    /// A map of PostgreSQL connection pools per named connection.
    postgres_pools: DashMap<String, sqlx::Pool<Postgres>>,

    /// A map of gRPC clients per named connection.
    grpc_clients: DashMap<String, resources::GrpcClient>,

    /// A map of Kafka producers per named connection.
    kafka_producers: DashMap<String, resources::KafkaProducer>,

    /// A map of file loggers per named connection.
    file_loggers: DashMap<String, resources::FileLogger>,

    /// The name of the extension.
    config: Arc<ExtensionConfig>,
}

impl WasiState {
    /// Creates a new instance of `WasiState` with the given WASI context.
    ///
    /// # Arguments
    ///
    /// * `ctx` - A `WasiCtx` instance that represents the WASI environment context.
    ///
    /// # Returns
    ///
    /// A new `WasiState` instance initialized with the provided context and default
    /// HTTP and resource table contexts.
    pub fn new(config: Arc<ExtensionConfig>, cache: Arc<Cache>) -> Self {
        let meter = meter_from_global_provider();
        let request_durations = meter.u64_histogram("grafbase.hook.http_request.duration").build();
        let http_client = reqwest::Client::new();

        Self {
            ctx: crate::config::build_context(&config.wasm),
            http_ctx: WasiHttpCtx::new(),
            table: ResourceTable::new(),
            request_durations,
            http_client,
            cache,
            postgres_pools: DashMap::new(),
            grpc_clients: DashMap::new(),
            kafka_producers: DashMap::new(),
            file_loggers: DashMap::new(),
            config,
        }
    }

    /// Pushes a resource into the shared memory, allowing it to be managed by the resource table.
    pub fn push_resource<T: Send + 'static>(&mut self, entry: T) -> crate::Result<Resource<T>> {
        Ok(self.table.push(entry).map_err(anyhow::Error::from)?)
    }

    /// Takes ownership of a resource identified by its representation ID from the shared memory.
    pub fn take_resource<T: 'static>(&mut self, rep: u32) -> crate::Result<T> {
        let resource = self
            .table
            .delete(Resource::<T>::new_own(rep))
            .map_err(anyhow::Error::from)?;

        Ok(resource)
    }

    /// Gets a mutable reference to the instance identified by the given resource.
    pub fn get_mut<T: 'static>(&mut self, resource: &Resource<T>) -> crate::Result<&mut T> {
        let entry = self.table.get_mut(resource).map_err(anyhow::Error::from)?;

        Ok(entry)
    }

    /// Retrieves a reference to the instance identified by the given resource.
    pub fn get<T: 'static>(&self, resource: &Resource<T>) -> crate::Result<&T> {
        let entry = self.table.get(resource).map_err(anyhow::Error::from)?;

        Ok(entry)
    }

    /// Returns a reference to the histogram tracking request durations.
    pub fn request_durations(&self) -> &Histogram<u64> {
        &self.request_durations
    }

    /// Returns a reference to the map of gRPC clients.
    pub(crate) fn grpc_clients(&self) -> &DashMap<String, GrpcClient> {
        &self.grpc_clients
    }

    /// Returns a reference to the map of file loggers.
    pub fn file_loggers(&self) -> &DashMap<String, FileLogger> {
        &self.file_loggers
    }

    /// Returns a reference to the HTTP client used for making requests from the guest.
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// Returns a reference to the map of PostgreSQL connection pools.
    pub fn postgres_pools(&self) -> &DashMap<String, sqlx::Pool<Postgres>> {
        &self.postgres_pools
    }

    /// Returns a reference to the map of Kafka producers.
    pub fn kafka_producers(&self) -> &DashMap<String, resources::KafkaProducer> {
        &self.kafka_producers
    }

    /// Returns a reference to the cache.
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    /// Returns whether network operations are enabled for this WASI instance.
    ///
    /// When `false`, any network operations attempted by the guest will fail.
    pub fn network_enabled(&self) -> bool {
        self.config.wasm.networking
    }

    pub fn extension_name(&self) -> &str {
        &self.config.manifest_id.name
    }

    pub fn extension_id(&self) -> ExtensionId {
        self.config.id
    }
}

impl IoView for WasiState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasiView for WasiState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

impl WasiHttpView for WasiState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http_ctx
    }
}
