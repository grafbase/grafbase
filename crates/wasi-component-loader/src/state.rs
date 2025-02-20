use std::sync::Arc;

use super::cache::Cache;
use grafbase_telemetry::{metrics::meter_from_global_provider, otel::opentelemetry::metrics::Histogram};
use wasmtime::component::Resource;
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

use crate::ChannelLogSender;

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
    table: ResourceTable,

    /// The histogram for request durations.
    request_durations: Histogram<u64>,

    /// A client for making HTTP requests from the guest.
    http_client: reqwest::Client,

    /// A sender for the access log channel.
    access_log: ChannelLogSender,

    /// A cache to be used for storing data between calls to different instances of the same extension.
    cache: Arc<Cache>,
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
    pub fn new(ctx: WasiCtx, access_log: ChannelLogSender, cache: Arc<Cache>) -> Self {
        let meter = meter_from_global_provider();
        let request_durations = meter.u64_histogram("grafbase.hook.http_request.duration").build();
        let http_client = reqwest::Client::new();

        Self {
            ctx,
            http_ctx: WasiHttpCtx::new(),
            table: ResourceTable::new(),
            request_durations,
            http_client,
            access_log,
            cache,
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

    /// Returns a reference to the HTTP client used for making requests from the guest.
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// Returns a reference to the access log sender.
    pub fn access_log(&self) -> &ChannelLogSender {
        &self.access_log
    }

    /// Returns a reference to the cache.
    pub fn cache(&self) -> &Cache {
        &self.cache
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
