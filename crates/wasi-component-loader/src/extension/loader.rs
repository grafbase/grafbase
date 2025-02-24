use std::sync::Arc;

use super::{instance::ExtensionInstance, wit};
use anyhow::Context;
use gateway_config::WasiExtensionsConfig;
use wasmtime::{
    Engine, Store,
    component::{Component, Linker},
};

use crate::{cache::Cache, config::build_extensions_context, state::WasiState};
pub struct ExtensionLoader {
    component_config: WasiExtensionsConfig,
    r#type: wit::ExtensionType,
    guest_config: Vec<u8>,
    #[allow(unused)] // MUST be unused, or at least immutable, we self-reference to it
    schema_directives: Vec<SchemaDirective>,
    // Self-reference to schema_directives
    wit_schema_directives: Vec<wit::Directive<'static>>,
    pre: wit::SdkPre<WasiState>,
    cache: Arc<Cache>,
    shared: crate::resources::SharedResources,
}

pub struct ExtensionGuestConfig<T> {
    pub r#type: extension_catalog::KindDiscriminants,
    pub schema_directives: Vec<SchemaDirective>,
    pub configuration: T,
}

pub struct SchemaDirective {
    name: String,
    subgraph_name: String,
    arguments: Vec<u8>,
}

impl SchemaDirective {
    pub fn new<T: serde::Serialize>(name: impl Into<String>, subgraph_name: impl Into<String>, args: T) -> Self {
        Self {
            name: name.into(),
            subgraph_name: subgraph_name.into(),
            arguments: minicbor_serde::to_vec(args).unwrap(),
        }
    }
}

impl ExtensionLoader {
    pub fn new<T>(
        shared: crate::resources::SharedResources,
        component_config: impl Into<WasiExtensionsConfig>,
        guest_config: ExtensionGuestConfig<T>,
    ) -> crate::Result<Self>
    where
        T: serde::Serialize,
    {
        let component_config: WasiExtensionsConfig = component_config.into();
        let mut engine_config = wasmtime::Config::new();

        engine_config.wasm_component_model(true);
        engine_config.async_support(true);

        let engine = Engine::new(&engine_config)?;
        let component = Component::from_file(&engine, &component_config.location)?;

        tracing::debug!(
            location = component_config.location.to_str(),
            "loaded the provided web assembly component successfully",
        );

        let mut linker = Linker::<WasiState>::new(&engine);

        // adds the wasi interfaces to our component
        wasmtime_wasi::add_to_linker_async(&mut linker)?;

        if component_config.networking {
            // adds the wasi http interfaces to our component
            wasmtime_wasi_http::add_only_http_to_linker_async(&mut linker)?;
        }

        wit::add_to_linker(&mut linker, |state| state)?;

        let instnace_pre = linker.instantiate_pre(&component)?;
        let pre = wit::SdkPre::<WasiState>::new(instnace_pre)?;
        let schema_directives = guest_config.schema_directives;
        let wit_schema_directives = schema_directives
            .iter()
            .map(|dir| {
                let dir = wit::Directive {
                    name: &dir.name,
                    subgraph_name: &dir.subgraph_name,
                    arguments: &dir.arguments,
                };
                // SAFETY: Self-reference to schema_directives which is kept alive and never
                // changed.
                let dir: wit::Directive<'static> = unsafe { std::mem::transmute(dir) };
                dir
            })
            .collect();

        Ok(Self {
            shared,
            component_config,
            r#type: guest_config.r#type.into(),
            guest_config: minicbor_serde::to_vec(&guest_config.configuration)
                .context("Could not serialize configuration")?,
            schema_directives,
            wit_schema_directives,
            pre,
            cache: Arc::new(Cache::new()),
        })
    }

    pub async fn instantiate(&self) -> crate::Result<ExtensionInstance> {
        let state = WasiState::new(
            build_extensions_context(&self.component_config),
            self.shared.access_log.clone(),
            self.cache.clone(),
        );
        let mut store = Store::new(self.pre.engine(), state);
        let inner = self.pre.instantiate_async(&mut store).await?;
        inner.call_register_extension(&mut store).await?;
        inner
            .grafbase_sdk_extension()
            .call_init_gateway_extension(&mut store, self.r#type, &self.wit_schema_directives, &self.guest_config)
            .await??;
        Ok(ExtensionInstance {
            store,
            inner,
            poisoned: false,
        })
    }
}
