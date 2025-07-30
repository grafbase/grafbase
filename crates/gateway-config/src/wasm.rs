use std::path::PathBuf;

#[derive(Default, Clone, Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WasmConfig {
    pub cache_path: Option<PathBuf>,
}
