use std::sync::Arc;

use super::{ExtensionConfig, ExtensionInstance, WasmConfig};
use crate::{cache::Cache, config::build_context, extension::api::SdkPre, state::WasiState};
use engine_schema::Schema;
use wasmtime::{
    CacheConfig, Engine,
    component::{Component, Linker},
};

pub(crate) struct ExtensionLoader {
    wasm_config: WasmConfig,
    pre: SdkPre,
    cache: Arc<Cache>,
    name: String,
}

impl ExtensionLoader {
    pub(crate) fn new<T: serde::Serialize>(schema: Arc<Schema>, config: ExtensionConfig<T>) -> crate::Result<Self> {
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
            wasm_config: config.wasm,
            pre,
            cache: Arc::new(Cache::new()),
            name: config.extension_name,
        })
    }

    pub async fn instantiate(&self) -> crate::Result<Box<dyn ExtensionInstance>> {
        let state = WasiState::new(
            build_context(&self.wasm_config),
            self.cache.clone(),
            self.wasm_config.networking,
            self.name.clone(),
        );

        self.pre.instantiate(state).await
    }
}
