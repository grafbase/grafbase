//! # Customer hooks with WebAssembly component model
//!
//! This crate provides library support to load and run custom code compiled as a [WebAssembly component].
//! The calling code in this crate is called "host" and the called code "guest".
//!
//! It is important the compiled WebAssembly code implements at least the minimal required types and interfaces.
//! More on those on the crate README.

#![deny(missing_docs)]

mod access_log;
mod cache;
mod config;
mod context;
mod error;
mod headers;
mod http_client;
mod instance;
mod names;
mod state;

#[cfg(test)]
mod tests;

use std::sync::Arc;

pub use access_log::{create_log_channel, AccessLogMessage, ChannelLogReceiver, ChannelLogSender};
use cache::Cache;
pub use config::{ExtensionsConfig, HooksWasiConfig};
pub use context::{ContextMap, SharedContext};
pub use crossbeam::channel::Sender;
pub use crossbeam::sync::WaitGroup;
use either::Either;
pub use error::{guest::GuestError, Error, GatewayError};
use gateway_config::WasiExtensionsConfig;
pub use instance::extensions::{Directive, ExtensionType, ExtensionsComponentInstance, FieldDefinition, FieldOutput};
pub use instance::hooks::{
    authorization::{EdgeDefinition, NodeDefinition},
    response::{
        CacheStatus, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest, FieldError,
        GraphqlResponseStatus, RequestError, SubgraphRequestExecutionKind, SubgraphResponse,
    },
    HookImplementation, HooksComponentInstance,
};

/// The crate result type
pub type Result<T> = std::result::Result<T, Error>;
/// The guest result type
pub type GuestResult<T> = std::result::Result<T, GuestError>;
/// The gateway result type
pub type GatewayResult<T> = std::result::Result<T, GatewayError>;

use state::WasiState;
use wasmtime::{
    component::{Component, Linker, LinkerInstance},
    Engine,
};

/// A structure responsible for loading and managing WebAssembly components.
///
/// The `ComponentLoader` is designed to facilitate the loading and execution of
/// WebAssembly components within the Wasmtime environment. It manages the
/// configuration, engine, linker, and the component itself, providing the necessary
/// interfaces for interaction with the loaded component.
pub struct ComponentLoader {
    /// The Wasmtime engine used for running the WebAssembly component.
    engine: Engine,
    /// The linker that connects the component to its dependencies.
    linker: Linker<WasiState>,
    /// The WebAssembly component being loaded.
    component: Component,
    /// Configuration settings for the component loader.
    config: Either<HooksWasiConfig, (String, WasiExtensionsConfig)>,
    /// Shared cache between component instances.
    cache: Arc<Cache>,
}

impl ComponentLoader {
    /// Creates a new instance of `ComponentLoader` for gateway hooks with the specified
    /// configuration.
    pub fn hooks(config: HooksWasiConfig) -> Result<Option<Self>> {
        let instantiate = |mut instance: LinkerInstance<'_, WasiState>| -> Result<()> {
            headers::inject_mapping(&mut instance)?;
            context::inject_mapping(&mut instance)?;
            context::inject_shared_mapping(&mut instance)?;
            http_client::inject_mapping(&mut instance)?;
            access_log::inject_mapping(&mut instance)?;

            Ok(())
        };

        Self::new(Either::Left(config), instantiate)
    }

    /// Creates a new instance of `ComponentLoader` for gateway extensions with the specified
    /// configuration.
    pub fn extensions(extension_name: String, config: impl Into<WasiExtensionsConfig>) -> Result<Option<Self>> {
        let instantiate = |mut instance: LinkerInstance<'_, WasiState>| -> Result<()> {
            headers::inject_mapping(&mut instance)?;
            context::inject_shared_mapping(&mut instance)?;
            http_client::inject_mapping(&mut instance)?;
            access_log::inject_mapping(&mut instance)?;
            cache::inject_mapping(&mut instance)?;

            Ok(())
        };

        Self::new(Either::Right((extension_name, config.into())), instantiate)
    }

    fn new<F>(config: Either<HooksWasiConfig, (String, WasiExtensionsConfig)>, instantiate: F) -> Result<Option<Self>>
    where
        F: FnOnce(LinkerInstance<'_, WasiState>) -> Result<()>,
    {
        let mut wasm_config = wasmtime::Config::new();

        wasm_config.wasm_component_model(true);
        wasm_config.async_support(true);

        let engine = Engine::new(&wasm_config)?;

        let (networking, location) = match config {
            Either::Left(ref hooks) => (hooks.networking, hooks.location.clone()),
            Either::Right((_, ref config)) => (config.networking, config.location.clone()),
        };

        let this = match Component::from_file(&engine, &location) {
            Ok(component) => {
                tracing::debug!(
                    location = location.to_str(),
                    "loaded the provided web assembly component successfully",
                );

                let mut linker = Linker::<WasiState>::new(&engine);

                // adds the wasi interfaces to our component
                wasmtime_wasi::add_to_linker_async(&mut linker)?;

                if networking {
                    // adds the wasi http interfaces to our component
                    wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;
                }

                instantiate(linker.root())?;

                Some(Self {
                    engine,
                    linker,
                    component,
                    config,
                    cache: Arc::new(Cache::new()),
                })
            }
            Err(e) => {
                tracing::error!(
                    location = location.to_str(),
                    "error loading web assembly component: {e}",
                );

                None
            }
        };

        Ok(this)
    }

    /// Returns a reference to the configuration settings for this component loader.
    ///
    /// This function provides access to the `Config` structure, which contains the
    /// configuration settings that were used to initialize the `ComponentLoader`.
    pub(crate) fn config(&self) -> &Either<HooksWasiConfig, (String, WasiExtensionsConfig)> {
        &self.config
    }

    /// Returns a reference to the Wasmtime engine used by this component loader.
    ///
    /// This function provides access to the `Engine` instance, which is responsible
    /// for running the WebAssembly component. The engine is initialized with the
    /// configuration settings specified during the creation of the `ComponentLoader`.
    pub(crate) fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Returns a reference to the linker used by this component loader.
    ///
    /// This function provides access to the `Linker<WasiState>` instance, which connects
    /// the loaded WebAssembly component to its dependencies, including the necessary WASI
    /// interfaces. It is essential for executing the component correctly within the Wasmtime
    /// environment.
    pub(crate) fn linker(&self) -> &Linker<WasiState> {
        &self.linker
    }

    /// Returns a reference to the loaded WebAssembly component.
    ///
    /// This function provides access to the `Component` instance, which represents
    /// the WebAssembly component that has been loaded into the `ComponentLoader`.
    /// It is used to interact with the component's exported functions and types.
    pub(crate) fn component(&self) -> &Component {
        &self.component
    }

    /// Checks if the WebAssembly component implements a specific interface.
    pub fn implements_interface(&self, interface_name: &'static str) -> bool {
        self.component.export_index(None, interface_name).is_some()
    }

    /// Shared cache between component instances.
    fn cache(&self) -> &Arc<Cache> {
        &self.cache
    }
}
