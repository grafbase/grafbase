use std::sync::Arc;

use engine_schema::Schema;
use extension_catalog::{ExtensionCatalog, TypeDiscriminants};
use gateway_config::Config;

use crate::resources::SharedResources;

use super::{ExtensionLoader, Pool, config};

#[derive(Clone)]
pub struct WasmHooks(Arc<WasiHooksInner>);

#[derive(Default)]
struct WasiHooksInner {
    pool: Option<Pool>,
}

impl WasmHooks {
    pub async fn new(
        shared_resources: &SharedResources,
        extension_catalog: &ExtensionCatalog,
        gateway_config: &Config,
    ) -> crate::Result<Self> {
        let extensions = config::load_extensions_config(extension_catalog, gateway_config);
        let mut selected_config = None;

        for config in extensions.into_iter() {
            if matches!(config.r#type, TypeDiscriminants::Hooks) {
                match selected_config {
                    None => {
                        selected_config = Some(config);
                    }
                    Some(ref selected_config) => {
                        tracing::warn!(
                            "detected multiple hooks extensions, using the previously selected extension '{}' and skipping '{}' since only one hooks extension can be loaded at a time",
                            selected_config.extension_name,
                            config.extension_name,
                        );
                    }
                }
            }
        }

        let pool = match selected_config {
            Some(config) => {
                let max_pool_size = config.pool.max_size;
                let schema = Schema::empty().await;

                let loader = ExtensionLoader::new(Arc::new(schema), shared_resources.clone(), config)?;

                Some(Pool::new(loader, max_pool_size))
            }
            None => None,
        };

        Ok(Self(Arc::new(WasiHooksInner { pool })))
    }

    pub(crate) fn pool(&self) -> Option<&Pool> {
        self.0.pool.as_ref()
    }
}
