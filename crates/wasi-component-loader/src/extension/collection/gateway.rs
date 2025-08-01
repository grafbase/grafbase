use std::sync::Arc;

use engine_schema::Schema;
use enumflags2::BitFlag;
use extension_catalog::{ExtensionCatalog, HooksType, TypeDiscriminants};
use gateway_config::Config;

use crate::{
    ExtensionState,
    extension::{Pool, engine::build_engine, load_extensions_config},
};

/// Extensions tied to the gateway, rather than the engine. As such they won't reload if the schema
/// changes.
#[derive(Default, Clone)]
pub struct GatewayWasmExtensions(Arc<GatewayWasmExtensionsInner>);

impl std::ops::Deref for GatewayWasmExtensions {
    type Target = Arc<GatewayWasmExtensionsInner>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct GatewayWasmExtensionsInner {
    pub(crate) engine: wasmtime::Engine,
    pub(crate) hooks: Option<Pool>,
    pub(crate) hooks_event_filter: Option<event_queue::EventFilter>,
    pub(crate) authentication: Vec<Pool>,
}

impl Default for GatewayWasmExtensionsInner {
    fn default() -> Self {
        let engine = build_engine(Default::default()).unwrap();
        Self {
            engine,
            hooks: None,
            hooks_event_filter: None,
            authentication: Vec::new(),
        }
    }
}

impl GatewayWasmExtensions {
    pub async fn new(
        extension_catalog: &ExtensionCatalog,
        gateway_config: &Config,
        logging_filter: String,
    ) -> wasmtime::Result<Self> {
        let engine = build_engine(gateway_config.wasm.clone().unwrap_or_default())?;

        let extension_configs = load_extensions_config(extension_catalog, gateway_config, logging_filter, |ty| {
            matches!(ty, TypeDiscriminants::Hooks | TypeDiscriminants::Authentication)
        });

        let mut inner = GatewayWasmExtensionsInner {
            engine,
            hooks: None,
            hooks_event_filter: None,
            authentication: Vec::new(),
        };

        // dummy schema as we use a common extension loader struct for all extensions.
        let schema = Arc::new(Schema::empty().await);
        for config in extension_configs {
            let manifiest = &extension_catalog[config.id].manifest;
            match &manifiest.r#type {
                extension_catalog::Type::Hooks(HooksType { event_filter }) => {
                    if inner.hooks.is_some() {
                        return Err(wasmtime::Error::msg(
                            "Multiple hooks extensions found in the configuration, but only one is allowed.",
                        ));
                    }
                    inner.hooks_event_filter = event_filter
                        .as_ref()
                        .or(manifiest.legacy_event_filter.as_ref())
                        .map(convert_event_filter);
                    inner.hooks = Some(Pool::new(&inner.engine, &schema, Arc::new(ExtensionState::new(config))).await?);
                }
                extension_catalog::Type::Authentication(_) => {
                    inner
                        .authentication
                        .push(Pool::new(&inner.engine, &schema, Arc::new(ExtensionState::new(config))).await?);
                }
                _ => continue,
            }
        }

        Ok(Self(Arc::new(inner)))
    }
}

fn convert_event_filter(filter: &extension_catalog::EventFilter) -> event_queue::EventFilter {
    match filter {
        extension_catalog::EventFilter::All => event_queue::EventFilter::All,
        extension_catalog::EventFilter::Types(types) => {
            let mut out = event_queue::EventFilterType::empty();
            for ty in types {
                out.insert(match ty {
                    extension_catalog::EventType::Operation => event_queue::EventFilterType::Operation,
                    extension_catalog::EventType::SubgraphRequest => event_queue::EventFilterType::SubgraphRequest,
                    extension_catalog::EventType::HttpRequest => event_queue::EventFilterType::HttpRequest,
                    extension_catalog::EventType::Extension => event_queue::EventFilterType::Extension,
                });
            }
            event_queue::EventFilter::Types(out)
        }
    }
}
