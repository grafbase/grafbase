//! # Customer hooks with WebAssembly component model
//!
//! This crate provides library support to load and run custom code compiled as a [WebAssembly component].
//! The calling code in this crate is called "host" and the called code "guest".
//!
//! It is important the compiled WebAssembly code implements at least the minimal required types and interfaces.
//! More on those on the crate README.

#![deny(missing_docs)]

use grafbase_workspace_hack as _;

mod config;
mod context;
mod error;
mod headers;
mod hooks;
mod names;
mod state;

#[cfg(test)]
mod tests;

pub use config::Config;
pub use context::{
    create_log_channel, AccessLogMessage, ChannelLogReceiver, ChannelLogSender, ContextMap, SharedContext,
};
pub use crossbeam::channel::Sender;
pub use crossbeam::sync::WaitGroup;
pub use error::{guest::GuestError, Error};
pub use hooks::{
    authorization::{AuthorizationComponentInstance, EdgeDefinition, NodeDefinition},
    gateway::GatewayComponentInstance,
    response::{
        CacheStatus, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest, FieldError,
        GraphqlResponseStatus, RequestError, ResponsesComponentInstance, SubgraphRequestExecutionKind,
        SubgraphResponse,
    },
    subgraph::*,
    RecycleableComponentInstance,
};

/// The crate result type
pub type Result<T> = std::result::Result<T, Error>;
/// The guest result type
pub type GuestResult<T> = std::result::Result<T, GuestError>;

use state::WasiState;
use wasmtime::{
    component::{Component, Linker},
    Engine,
};

use crate::names::COMPONENT_TYPES;

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
    config: Config,
}

impl ComponentLoader {
    /// Creates a new instance of `ComponentLoader` with the specified configuration.
    ///
    /// This function initializes the Wasmtime engine and linker, loads the WebAssembly
    /// component from the specified location in the configuration, and sets up the necessary
    /// WASI interfaces. If the component is loaded successfully, it returns an instance of
    /// `ComponentLoader`; otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// - `config`: The configuration settings for the component loader.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing an `Option<Self>`. The `Option` will be `Some` if the
    /// component is loaded successfully, or `None` if there was an error during loading.
    pub fn new(config: Config) -> Result<Option<Self>> {
        let mut wasm_config = wasmtime::Config::new();

        // Read more on WebAssembly component model:
        // https://component-model.bytecodealliance.org/
        wasm_config.wasm_component_model(true);

        // Read more on Wasmtime async functions and fuel consumption:
        // https://docs.rs/wasmtime/latest/wasmtime/struct.Config.html#method.async_support
        wasm_config.async_support(true);
        wasm_config.consume_fuel(true);

        // https://github.com/bytecodealliance/wasmtime/issues/8897
        #[cfg(not(target_os = "windows"))]
        wasm_config.native_unwind_info(false);

        let engine = Engine::new(&wasm_config)?;

        let this = match Component::from_file(&engine, &config.location) {
            Ok(component) => {
                tracing::debug!(
                    location = config.location.to_str(),
                    "loaded the provided web assembly component successfully",
                );

                let mut linker = Linker::<WasiState>::new(&engine);

                // adds the wasi interfaces to our component
                wasmtime_wasi::add_to_linker_async(&mut linker)?;

                if config.networking {
                    // adds the wasi http interfaces to our component
                    wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;
                }

                let mut types = linker.instance(COMPONENT_TYPES)?;

                headers::map(&mut types)?;
                context::map(&mut types)?;
                context::map_shared(&mut types)?;

                Some(Self {
                    engine,
                    linker,
                    component,
                    config,
                })
            }
            Err(e) => {
                tracing::error!(
                    location = config.location.to_str(),
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
    pub(crate) fn config(&self) -> &Config {
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
}
