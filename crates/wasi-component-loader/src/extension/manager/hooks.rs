use std::sync::Arc;

use engine_schema::Schema;
use extension_catalog::{Extension, ExtensionId};
use gateway_config::Config;
use strum::IntoDiscriminant;

use super::{ExtensionConfig, ExtensionLoader, Pool, PoolConfig, WasmConfig};

#[derive(Clone)]
pub struct WasmHooks(Arc<WasiHooksInner>);

#[derive(Default)]
struct WasiHooksInner {
    pool: Option<Pool>,
    extension: Option<Extension>,
}

impl WasmHooks {
    pub async fn new(gateway_config: &Config, extension: Option<Extension>) -> crate::Result<Self> {
        let Some(extension) = extension else {
            return Ok(Self(Arc::new(WasiHooksInner {
                pool: None,
                extension: None,
            })));
        };

        let mut selected_config = None;

        for (name, config) in gateway_config.extensions.iter() {
            if name != extension.manifest.name() {
                continue;
            }

            selected_config = Some(config);
        }

        let extension_config = ExtensionConfig {
            id: ExtensionId::from(0_u16),
            manifest_id: extension.manifest.id.clone(),
            r#type: extension.manifest.r#type.discriminant(),
            sdk_version: extension.manifest.sdk_version.clone(),
            pool: PoolConfig {
                max_size: selected_config.and_then(|c| c.max_pool_size()),
            },
            wasm: WasmConfig {
                location: extension.wasm_path.clone(),
                networking: extension.manifest.network_enabled(),
                stdout: extension.manifest.stdout_enabled(),
                stderr: extension.manifest.stderr_enabled(),
                environment_variables: extension.manifest.environment_variables_enabled(),
            },
            guest_config: selected_config.and_then(|c| c.config().cloned()),
        };

        let max_pool_size = extension_config.pool.max_size;
        let schema = Schema::empty().await;
        let loader = ExtensionLoader::new(Arc::new(schema), extension_config)?;
        let pool = Pool::new(loader, max_pool_size);

        Ok(Self(Arc::new(WasiHooksInner {
            pool: Some(pool),
            extension: Some(extension),
        })))
    }

    pub(crate) fn pool(&self) -> Option<&Pool> {
        self.0.pool.as_ref()
    }

    pub(crate) fn extension(&self) -> &Option<Extension> {
        &self.0.extension
    }
}
