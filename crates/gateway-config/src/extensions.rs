use std::path::PathBuf;

#[derive(PartialEq, serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ExtensionsConfig {
    Version(String),
    Structured(StructuredExtensionsConfig),
}

#[derive(PartialEq, serde::Deserialize, Debug, Clone)]
pub struct StructuredExtensionsConfig {
    pub version: String,
    pub networking: bool,
    pub stdout: bool,
    pub stderr: bool,
    pub environment_variables: bool,
    pub max_pool_size: usize,
}

#[derive(Debug, Clone)]
pub struct WasiExtensionsConfig {
    pub location: PathBuf,
    pub networking: bool,
    pub stdout: bool,
    pub stderr: bool,
    pub environment_variables: bool,
    pub max_pool_size: Option<usize>,
}

impl From<(String, ExtensionsConfig)> for WasiExtensionsConfig {
    fn from((name, config): (String, ExtensionsConfig)) -> Self {
        match config {
            ExtensionsConfig::Version(version) => {
                let location = std::env::current_dir()
                    .unwrap()
                    .join("grafbase_extensions")
                    .join(format!("{name}-{version}"))
                    .join("extension.wasm");

                Self {
                    location,
                    networking: false,
                    stdout: false,
                    stderr: false,
                    environment_variables: false,
                    max_pool_size: None,
                }
            }
            ExtensionsConfig::Structured(config) => {
                let location = std::env::current_dir()
                    .unwrap()
                    .join("grafbase_extensions")
                    .join(format!("{name}-{}", config.version))
                    .join("extension.wasm");

                Self {
                    location,
                    networking: config.networking,
                    stdout: config.stdout,
                    stderr: config.stderr,
                    environment_variables: config.environment_variables,
                    max_pool_size: Some(config.max_pool_size),
                }
            }
        }
    }
}
