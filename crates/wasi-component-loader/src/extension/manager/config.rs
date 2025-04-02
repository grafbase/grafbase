use std::path::PathBuf;

use extension_catalog::{ExtensionCatalog, ExtensionId};
use gateway_config::Config;
use semver::Version;

pub(crate) struct ExtensionConfig<T = toml::Value> {
    pub id: ExtensionId,
    pub manifest_id: extension_catalog::Id,
    pub r#type: extension_catalog::TypeDiscriminants,
    pub sdk_version: Version,
    pub pool: PoolConfig,
    pub wasm: WasmConfig,
    pub guest_config: Option<T>,
}

#[derive(Default)]
pub(crate) struct PoolConfig {
    pub max_size: Option<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct WasmConfig {
    pub location: PathBuf,
    pub networking: bool,
    pub stdout: bool,
    pub stderr: bool,
    pub environment_variables: bool,
}

pub(super) fn load_extensions_config(extension_catalog: &ExtensionCatalog, config: &Config) -> Vec<ExtensionConfig> {
    let mut wasm_extensions = Vec::with_capacity(extension_catalog.len());

    for (id, extension) in extension_catalog.iter_with_id() {
        let manifest = &extension.manifest;
        let extension_config = config
            .extensions
            .get(manifest.name())
            .expect("we made sure in the create_extension_catalog that this extension is in the config");

        let wasi_config = WasmConfig {
            location: extension.wasm_path.clone(),
            networking: extension_config.networking().unwrap_or(manifest.network_enabled()),
            stdout: extension_config.stdout().unwrap_or(manifest.stdout_enabled()),
            stderr: extension_config.stderr().unwrap_or(manifest.stderr_enabled()),
            environment_variables: extension_config
                .environment_variables()
                .unwrap_or(manifest.environment_variables_enabled()),
        };

        let max_pool_size = extension_config.max_pool_size();
        wasm_extensions.push(ExtensionConfig {
            id,
            manifest_id: manifest.id.clone(),
            r#type: manifest.r#type.clone(),
            pool: PoolConfig {
                max_size: max_pool_size,
            },
            wasm: wasi_config,
            guest_config: extension_config.config().cloned(),
            sdk_version: manifest.sdk_version.clone(),
        });
    }

    wasm_extensions
}
