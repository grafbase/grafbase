use std::sync::Arc;

use super::ExtensionInstance;
use crate::{ExtensionState, extension::api::SdkPre, state::InstanceState};
use engine_schema::Schema;
use wasmtime::{
    Engine,
    component::{Component, Linker},
};

pub(crate) struct ExtensionLoader {
    pre: SdkPre,
    state: Arc<ExtensionState>,
}

impl ExtensionLoader {
    pub(crate) fn new(engine: &Engine, schema: Arc<Schema>, state: Arc<ExtensionState>) -> wasmtime::Result<Self> {
        let cfg = &state.config.wasm;
        let component = Component::from_file(engine, &cfg.location)?;

        tracing::debug!(
            location = cfg.location.to_str(),
            "loaded the provided web assembly component successfully",
        );

        let mut linker = Linker::<InstanceState>::new(engine);

        // adds the wasi interfaces to our component
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

        if cfg.networking {
            // adds the wasi http interfaces to our component
            wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;
        }

        let pre = SdkPre::new(schema, &state.config, component, linker)?;

        Ok(Self { pre, state })
    }

    pub async fn instantiate(&self) -> wasmtime::Result<Box<dyn ExtensionInstance>> {
        let state = InstanceState::new(self.state.clone());
        self.pre.instantiate(state).await
    }
}
