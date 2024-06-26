//! # Customer hooks with WebAssembly component model
//!
//! This crate provides library support to load and run custom code compiled as a [WebAssembly component].
//! The calling code in this crate is called "host" and the called code "guest".
//!
//! It is important the compiled WebAssembly code implements at least the minimal required types and interfaces.
//! More on those on the crate README.

#![deny(missing_docs)]

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
pub use context::{ContextMap, SharedContextMap};
pub use error::{Error, ErrorResponse};

/// The crate result type
pub type Result<T> = std::result::Result<T, Error>;

use grafbase_tracing::span::GRAFBASE_TARGET;
use hooks::{authorization::AuthorizationHookInstance, gateway::GatewayHookInstance};
use http::HeaderMap;
use state::WasiState;
use wasmtime::{
    component::{Component, Linker},
    Engine,
};

use crate::names::COMPONENT_TYPES;

/// A loader for Grafbase WASI components. This is supposed to be reused for
/// the whole lifetime of the Grafbase Gateway.
pub struct ComponentLoader {
    engine: Engine,
    linker: Linker<WasiState>,
    component: Component,
    config: Config,
}

impl ComponentLoader {
    /// Initialize a new loader with the given config.
    pub fn new(config: Config) -> Result<Option<Self>> {
        let mut wasm_config = wasmtime::Config::new();

        // Read more on WebAssembly component model:
        // https://component-model.bytecodealliance.org/
        wasm_config.wasm_component_model(true);

        // Read more on Wasmtime async functions and fuel consumption:
        // https://docs.rs/wasmtime/latest/wasmtime/struct.Config.html#method.async_support
        wasm_config.async_support(true);
        wasm_config.consume_fuel(true);

        let engine = Engine::new(&wasm_config)?;

        let this = match Component::from_file(&engine, config.location()) {
            Ok(component) => {
                tracing::info!(target: GRAFBASE_TARGET, "loaded the provided WASM component successfully");

                let mut linker = Linker::<WasiState>::new(&engine);

                // adds the wasi interfaces to our component
                wasmtime_wasi::add_to_linker_async(&mut linker)?;

                if config.networking_enabled() {
                    // adds the wasi http interfaces to our component
                    wasmtime_wasi_http::proxy::add_only_http_to_linker(&mut linker)?;
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
                tracing::debug!(target: GRAFBASE_TARGET, "error loading WASM component: {e}");

                None
            }
        };

        Ok(this)
    }

    /// Initialize a new instance to trigger hooks from the gateway level. This should be called
    /// separately for every request to reset the state of the store, enabling the same amount of fuel
    /// for every request.
    ///
    /// Calls the user-defined hook from the guest, if the function is defined.
    pub async fn on_gateway_request(&self, context: ContextMap, headers: HeaderMap) -> Result<(ContextMap, HeaderMap)> {
        let hook = GatewayHookInstance::new(self, context, headers).await?;
        hook.call().await
    }

    /// Initialize a new instance to trigger hooks per authorization directive. This should be called
    /// separately for every request to reset the state of the store, enabling the same amount of fuel
    /// for every request.
    ///
    /// Calls the user-defined hook from the guest, if the function is defined.
    pub async fn authorized(
        &self,
        context: SharedContextMap,
        rule: String,
        input: Vec<String>,
    ) -> Result<Vec<Option<ErrorResponse>>> {
        let hook = AuthorizationHookInstance::new(self, context).await?;
        hook.call(rule, input).await
    }

    pub(crate) fn config(&self) -> &Config {
        &self.config
    }

    pub(crate) fn engine(&self) -> &Engine {
        &self.engine
    }

    pub(crate) fn linker(&self) -> &Linker<WasiState> {
        &self.linker
    }

    pub(crate) fn component(&self) -> &Component {
        &self.component
    }
}
