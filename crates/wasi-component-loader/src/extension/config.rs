use std::path::PathBuf;

use extension_catalog::{ExtensionCatalog, ExtensionId, HooksType};
use gateway_config::Config;
use semver::Version;

pub(crate) struct ExtensionConfig<T = toml::Value> {
    pub id: ExtensionId,
    pub manifest_id: extension_catalog::Id,
    pub r#type: extension_catalog::TypeDiscriminants,
    pub sdk_version: Version,
    pub pool: PoolConfig,
    pub wasm: WasmConfig,
    pub guest_config: T,
    pub can_skip_sending_events: bool,
    pub logging_filter: String,
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

pub(super) fn load_extensions_config(
    extension_catalog: &ExtensionCatalog,
    config: &Config,
    logging_filter: String,
    filter: impl Fn(extension_catalog::TypeDiscriminants) -> bool,
) -> Vec<ExtensionConfig> {
    let mut wasm_extensions = Vec::with_capacity(extension_catalog.len());

    let can_skip_sending_events = extension_catalog.iter().all(|ext| match &ext.manifest.r#type {
        extension_catalog::Type::Hooks(HooksType { event_filter }) => event_filter
            .as_ref()
            .or(ext.manifest.legacy_event_filter.as_ref())
            .map(|event_filter| match event_filter {
                extension_catalog::EventFilter::All => false,
                extension_catalog::EventFilter::Types(event_types) => {
                    event_types.contains(&extension_catalog::EventType::Extension)
                }
            })
            .unwrap_or(true),
        _ => true,
    });

    for (id, extension) in extension_catalog.iter_with_id() {
        let manifest = &extension.manifest;
        let r#type = manifest.r#type.clone().into();
        if !filter(r#type) {
            continue;
        }

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

        let max_size = extension_config.max_pool_size();

        wasm_extensions.push(ExtensionConfig {
            id,
            manifest_id: manifest.id.clone(),
            r#type,
            pool: PoolConfig { max_size },
            wasm: wasi_config,
            guest_config: extension_config
                .config()
                .cloned()
                .unwrap_or_else(|| toml::Value::Table(Default::default())),
            sdk_version: manifest.sdk_version.clone(),
            can_skip_sending_events,
            logging_filter: logging_filter.clone(),
        });
    }

    wasm_extensions
}
