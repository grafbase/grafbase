use std::sync::Arc;

use dashmap::DashMap;
use engine_error::{ErrorCode, ErrorResponse};
use extension_catalog::{ExtensionCatalog, ExtensionId};
use grafbase_telemetry::{metrics::meter_from_global_provider, otel::opentelemetry::metrics::Histogram};
use sqlx::Postgres;
use wasmtime::component::Resource;
use wasmtime_wasi::{
    ResourceTable,
    p2::{IoView, WasiCtx, WasiView},
};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

use crate::{
    cache::LegacyCache,
    extension::{ExtensionConfig, api::since_0_17_0::world as wit17, api::wit},
    resources::{Cache, FileLogger, GrpcClient, KafkaProducer, OwnedOrShared, WasmOwnedOrLease},
};

/// Represents the state of the WASI environment.
///
/// This structure encapsulates the WASI context, HTTP context, and a resource table
/// for managing shared resources in memory. It provides methods to create new instances,
/// manage resources, and access the contexts.
pub(crate) struct InstanceState {
    /// The WASI context that contains the state for the WASI environment.
    pub wasi_ctx: WasiCtx,

    /// The WASI HTTP context that handles HTTP-related operations.
    pub wasi_http_ctx: WasiHttpCtx,

    /// The resource table that manages shared resources in memory.
    pub resources: ResourceTable,

    pub shared: Arc<ExtensionState>,
}

impl std::ops::Deref for InstanceState {
    type Target = ExtensionState;
    fn deref(&self) -> &Self::Target {
        &self.shared
    }
}

/// Shared across extension instances and schema contracts.
pub(crate) struct ExtensionState {
    pub catalog: Arc<ExtensionCatalog>,

    /// The histogram for request durations.
    pub request_durations: Histogram<u64>,

    /// A client for making HTTP requests from the guest.
    pub http_client: reqwest::Client,

    /// A cache to be used for storing data between calls to different instances of the same extension.
    pub legacy_cache: LegacyCache, // Up to SDK 0.18
    pub caches: DashMap<String, Cache>, // Cache by name

    /// A map of PostgreSQL connection pools per named connection.
    pub postgres_pools: DashMap<String, sqlx::Pool<Postgres>>,

    /// A map of gRPC clients per named connection.
    pub grpc_clients: DashMap<String, GrpcClient>,

    /// A map of Kafka producers per named connection.
    pub kafka_producers: DashMap<String, KafkaProducer>,

    /// A map of file loggers per named connection.
    pub file_loggers: DashMap<String, FileLogger>,

    /// The name of the extension.
    pub config: ExtensionConfig,
}

impl ExtensionState {
    pub fn new(catalog: &Arc<ExtensionCatalog>, config: ExtensionConfig) -> Self {
        let meter = meter_from_global_provider();
        let request_durations = meter.u64_histogram("grafbase.hook.http_request.duration").build();
        let http_client = reqwest::Client::new();
        Self {
            catalog: catalog.clone(),
            request_durations,
            http_client,
            legacy_cache: LegacyCache::new(),
            caches: DashMap::new(),
            postgres_pools: DashMap::new(),
            grpc_clients: DashMap::new(),
            kafka_producers: DashMap::new(),
            file_loggers: DashMap::new(),
            config,
        }
    }
}

impl InstanceState {
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
    pub fn new(shared: Arc<ExtensionState>) -> Self {
        Self {
            wasi_ctx: crate::config::build_context(&shared.config.wasm),
            wasi_http_ctx: WasiHttpCtx::new(),
            resources: ResourceTable::new(),
            shared,
        }
    }

    /// Takes ownership of a leased resource that the guest cannot drop and doesn't have ownership
    /// of. Ideally we don't have any... Prefer sending the resource and returning it from the SDK.
    /// If not careful with it in the guest, this can lead to resources being dropped on the host
    /// side but not in the guest.
    pub fn take_leased_resource<T: 'static>(&mut self, rep: u32) -> wasmtime::Result<OwnedOrShared<T>> {
        let resource = self.resources.delete(Resource::<WasmOwnedOrLease<T>>::new_own(rep))?;
        Ok(resource.into_lease().unwrap())
    }

    pub fn take_error_response_sdk17(
        &mut self,
        err: wit17::ErrorResponse,
        code: ErrorCode,
    ) -> wasmtime::Result<ErrorResponse> {
        let headers = if let Some(resource) = err.headers {
            self.resources.delete(resource)?.into_inner().expect("Should be owned")
        } else {
            Default::default()
        };

        Ok(ErrorResponse::new(
            http::StatusCode::from_u16(err.status_code).unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR),
        )
        .with_errors(err.errors.into_iter().map(|err| err.into_graphql_error(code)))
        .with_headers(headers))
    }

    pub fn take_error_response(&mut self, err: wit::ErrorResponse, code: ErrorCode) -> wasmtime::Result<ErrorResponse> {
        let headers = if let Some(resource) = err.headers {
            self.resources.delete(resource)?.into_inner().expect("Should be owned")
        } else {
            Default::default()
        };

        Ok(ErrorResponse::new(
            http::StatusCode::from_u16(err.status_code).unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR),
        )
        .with_errors(err.errors.into_iter().map(|err| err.into_graphql_error(code)))
        .with_headers(headers))
    }

    /// Returns whether network operations are enabled for this WASI instance.
    ///
    /// When `false`, any network operations attempted by the guest will fail.
    pub fn is_network_enabled(&self) -> bool {
        self.config.wasm.networking
    }

    pub fn extension_name(&self) -> &str {
        &self.config.manifest_id.name
    }

    pub fn extension_id(&self) -> ExtensionId {
        self.config.id
    }
}

impl IoView for InstanceState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resources
    }
}

impl WasiView for InstanceState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}

impl WasiHttpView for InstanceState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.wasi_http_ctx
    }
}
