use std::path::PathBuf;

use engine_schema::Schema;
use extension_catalog::{ExtensionCatalog, ExtensionId};
use gateway_config::Config;
use semver::Version;

use crate::extension::SchemaDirective;

pub struct ExtensionConfig<T = toml::Value> {
    pub id: ExtensionId,
    pub manifest_id: extension_catalog::Id,
    pub sdk_version: Version,
    pub pool: PoolConfig,
    pub wasm: WasmConfig,
    pub schema_directives: Vec<SchemaDirective>,
    pub guest_config: Option<T>,
}

#[derive(Default)]
pub struct PoolConfig {
    pub max_size: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct WasmConfig {
    pub location: PathBuf,
    pub networking: bool,
    pub stdout: bool,
    pub stderr: bool,
    pub environment_variables: bool,
}

pub(super) fn load_extensions_config(
    extension_catalog: &ExtensionCatalog,
    config: &Config,
    schema: &Schema,
) -> Vec<ExtensionConfig> {
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
            pool: PoolConfig {
                max_size: max_pool_size,
            },
            wasm: wasi_config,
            schema_directives: Vec::new(),
            guest_config: extension_config.config().cloned(),
            sdk_version: manifest.sdk_version.clone(),
        });
    }

    for subgraph in schema.subgraphs() {
        let directives = subgraph.extension_schema_directives();

        for schema_directive in directives {
            let config = &mut wasm_extensions[usize::from(schema_directive.extension_id)];

            config.schema_directives.push(SchemaDirective::new(
                schema_directive.name(),
                subgraph.name(),
                schema_directive.static_arguments(),
            ));
        }
    }

    wasm_extensions
}
