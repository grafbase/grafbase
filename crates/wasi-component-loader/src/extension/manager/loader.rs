use std::sync::Arc;

use super::{ExtensionConfig, ExtensionInstance, WasmConfig};
use crate::{
    cache::Cache, config::build_extensions_context, extension::api::SdkPre, resources::SharedResources,
    state::WasiState,
};
use engine_schema::Schema;
use wasmtime::{
    CacheConfig, Engine,
    component::{Component, Linker},
};

pub(crate) struct ExtensionLoader {
    wasm_config: WasmConfig,
    pre: SdkPre,
    cache: Arc<Cache>,
    shared: SharedResources,
}

impl ExtensionLoader {
    pub(crate) fn new<T: serde::Serialize>(
        schema: Arc<Schema>,
        shared: SharedResources,
        config: ExtensionConfig<T>,
    ) -> crate::Result<Self> {
        let mut wasm_config = wasmtime::Config::new();

        wasm_config.wasm_component_model(true).async_support(true).cache(
            config
                .wasm
                .location
                .parent()
                // TODO: Properly expose Wasm cache. This is just a hack for our extensive
                // extension tests suite... Wasmtime create a thread per cache, so we should just
                // have one. In extensions test this doesn't matter as there is only one extension
                // anyway.
                .filter(|_| std::env::var("LOCAL_EXTENSION_WASM_CACHE").is_ok())
                .and_then(|parent| {
                    let dir = parent.join("cache");
                    if std::fs::create_dir(&dir).is_ok() || std::fs::read_dir(&dir).is_ok() {
                        let mut cfg = CacheConfig::new();
                        cfg.with_directory(dir);
                        wasmtime::Cache::new(cfg).ok()
                    } else {
                        None
                    }
                }),
        );

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
            shared,
            wasm_config: config.wasm,
            pre,
            cache: Arc::new(Cache::new()),
        })
    }

    pub async fn instantiate(&self) -> crate::Result<Box<dyn ExtensionInstance>> {
        let state = WasiState::new(
            build_extensions_context(&self.wasm_config),
            self.shared.access_log.clone(),
            self.cache.clone(),
            self.wasm_config.networking,
        );

        self.pre.instantiate(state).await
    }
}
