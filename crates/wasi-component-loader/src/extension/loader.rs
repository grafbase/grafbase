use std::sync::Arc;

use super::{ExtensionConfig, ExtensionInstance};
use crate::{cache::Cache, extension::api::SdkPre, state::WasiState};
use engine_schema::Schema;
use wasmtime::{
    CacheConfig, Engine,
    component::{Component, Linker},
};

pub(crate) struct ExtensionLoader {
    pub config: Arc<ExtensionConfig>,
    pre: SdkPre,
    cache: Arc<Cache>,
}

impl ExtensionLoader {
    pub(crate) fn new(schema: Arc<Schema>, config: Arc<ExtensionConfig>) -> wasmtime::Result<Self> {
        let mut wasm_config = wasmtime::Config::new();

        wasm_config
            .wasm_component_model(true)
            .async_support(true)
            .cache(config.wasm.location.parent().and_then(|parent| {
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
        let component = Component::from_file(&engine, &config.wasm.location)?;

        tracing::debug!(
            location = config.wasm.location.to_str(),
            "loaded the provided web assembly component successfully",
        );

        let mut linker = Linker::<WasiState>::new(&engine);

        // adds the wasi interfaces to our component
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

        if config.wasm.networking {
            // adds the wasi http interfaces to our component
            wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;
        }

        let pre = SdkPre::new(schema, &config, component, linker)?;

        Ok(Self {
            config,
            pre,
            cache: Arc::new(Cache::new()),
        })
    }

    pub async fn instantiate(&self) -> wasmtime::Result<Box<dyn ExtensionInstance>> {
        let state = WasiState::new(self.config.clone(), self.cache.clone());
        self.pre.instantiate(state).await
    }
}
