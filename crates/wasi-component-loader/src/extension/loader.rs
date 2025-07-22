use std::sync::Arc;

use super::ExtensionInstance;
use crate::{ExtensionState, extension::api::SdkPre, state::InstanceState};
use engine_schema::Schema;
use wasmtime::{
    CacheConfig, Engine,
    component::{Component, Linker},
};

pub(crate) struct ExtensionLoader {
    pre: SdkPre,
    state: Arc<ExtensionState>,
}

impl ExtensionLoader {
    pub(crate) fn new(schema: Arc<Schema>, state: Arc<ExtensionState>) -> wasmtime::Result<Self> {
        let mut wasm_config = wasmtime::Config::new();

        let cfg = &state.config.wasm;
        wasm_config
            .wasm_component_model(true)
            .async_support(true)
            .cache(cfg.location.parent().and_then(|parent| {
                let dir = parent.join("cache");
                if std::fs::create_dir(&dir).is_ok() || std::fs::read_dir(&dir).is_ok() {
                    let mut cfg = CacheConfig::new();
                    cfg.with_directory(dir);
                    wasmtime::Cache::new(cfg).ok()
                } else {
                    None
                }
            }));

        let engine = Engine::new(&wasm_config)?;
        let component = Component::from_file(&engine, &cfg.location)?;

        tracing::debug!(
            location = cfg.location.to_str(),
            "loaded the provided web assembly component successfully",
        );

        let mut linker = Linker::<InstanceState>::new(&engine);

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
