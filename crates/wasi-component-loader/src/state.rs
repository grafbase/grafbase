use std::{any::Any, sync::Arc};

use super::cache::Cache;
use grafbase_telemetry::{metrics::meter_from_global_provider, otel::opentelemetry::metrics::Histogram};
use wasmtime::component::Resource;
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

use crate::AccessLogSender;

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

    /// A sender for the access log channel.
    access_log: AccessLogSender,

    /// A cache to be used for storing data between calls to different instances of the same extension.
    cache: Arc<Cache>,

    /// If false, network operations are disabled.
    network_enabled: bool,
}

// Allows to define method for a resource that is either owned or an attribute from another one.
// In the later case we need the parent resource id and a method to retrieve our data.
// This is for SubgraphRequest and Headers typically, the former holds the actual http::HeaderMap,
// so for the guest to access it, we provide a "Ref" variant, but in other places we provide an
// "Owned" variant instead. For the guest, it's transparent.
pub enum WasmOwnedOrBorrowed<T> {
    /// Borrowed within the guest, typically accessing a resource from another resource.
    GuestBorrowed {
        parent: u32,
        get: for<'a> fn(elem: &'a mut (dyn Any + 'static)) -> &'a mut T,
    },
    /// Borrowed from the host, the instance will be provided with T, but it should not be dropped.
    /// The caller will remove it himself from the store.
    HostBorrowed(T),
    /// Fully owned by the guest
    Owned(T),
}

impl<T> WasmOwnedOrBorrowed<T> {
    pub fn borrow(data: T) -> Self {
        Self::HostBorrowed(data)
    }

    pub fn is_guest_borrowed(&self) -> bool {
        matches!(self, Self::GuestBorrowed { .. })
    }

    pub fn is_host_borrowed(&self) -> bool {
        matches!(self, Self::HostBorrowed(_))
    }

    pub fn into_owned(self) -> Option<T> {
        match self {
            Self::Owned(data) | Self::HostBorrowed(data) => Some(data),
            _ => None,
        }
    }
}

// Simple macro to create a ref resource for a field of an existing resource.
macro_rules! get_child_ref {
    ($store: ident, $resource: ident: $rty: ty => $field: ident: $ty: ty) => {{
        let store = $store.data_mut();

        let _ = store.get(&$resource)?;

        fn get(elem: &mut dyn std::any::Any) -> &mut $ty {
            &mut elem.downcast_mut::<$rty>().unwrap().$field
        }

        let field_ref = store.table.push_child(
            $crate::WasmOwnedOrBorrowed::GuestBorrowed {
                parent: $resource.rep(),
                get,
            },
            &$resource,
        )?;

        field_ref
    }};
}

pub(crate) use get_child_ref;

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
    pub fn new(ctx: WasiCtx, access_log: AccessLogSender, cache: Arc<Cache>, network_enabled: bool) -> Self {
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
            network_enabled,
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

    pub fn get_ref_mut<T: 'static>(&mut self, resource: &Resource<WasmOwnedOrBorrowed<T>>) -> crate::Result<&mut T> {
        let data = self.table.get(resource).map_err(anyhow::Error::from)?;

        if let WasmOwnedOrBorrowed::GuestBorrowed { parent, get } = *data {
            let data = self.table.get_any_mut(parent).map_err(anyhow::Error::from)?;
            return Ok(get(data));
        }

        match self.table.get_mut(resource).map_err(anyhow::Error::from)? {
            WasmOwnedOrBorrowed::Owned(data) | WasmOwnedOrBorrowed::HostBorrowed(data) => Ok(data),
            _ => unreachable!(),
        }
    }

    pub fn get_ref<T: 'static>(&mut self, resource: &Resource<WasmOwnedOrBorrowed<T>>) -> crate::Result<&T> {
        let data = self.table.get(resource).map_err(anyhow::Error::from)?;

        if let WasmOwnedOrBorrowed::GuestBorrowed { parent, get } = *data {
            let data = self.table.get_any_mut(parent).map_err(anyhow::Error::from)?;
            return Ok(get(data));
        }

        match self.table.get(resource).map_err(anyhow::Error::from)? {
            WasmOwnedOrBorrowed::Owned(data) | WasmOwnedOrBorrowed::HostBorrowed(data) => Ok(data),
            _ => unreachable!(),
        }
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
    pub fn access_log(&self) -> &AccessLogSender {
        &self.access_log
    }

    /// Returns a reference to the cache.
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    /// Returns whether network operations are enabled for this WASI instance.
    ///
    /// When `false`, any network operations attempted by the guest will fail.
    pub fn network_enabled(&self) -> bool {
        self.network_enabled
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
