use std::path::{Path, PathBuf};

use semver::VersionReq;

#[derive(PartialEq, serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ExtensionsConfig {
    Version(VersionReq),
    Structured(StructuredExtensionsConfig),
}

#[derive(PartialEq, serde::Deserialize, Debug, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct StructuredExtensionsConfig {
    pub version: VersionReq,
    pub path: Option<PathBuf>,
    pub networking: bool,
    pub stdout: bool,
    pub stderr: bool,
    pub environment_variables: bool,
    pub max_pool_size: Option<usize>,
}

impl Default for StructuredExtensionsConfig {
    fn default() -> Self {
        Self {
            version: VersionReq::parse("*").unwrap(),
            path: None,
            networking: false,
            stdout: false,
            stderr: false,
            environment_variables: false,
            max_pool_size: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WasiExtensionsConfig {
    pub location: PathBuf,
    pub networking: bool,
    pub stdout: bool,
    pub stderr: bool,
    pub environment_variables: bool,
}

impl ExtensionsConfig {
    pub fn version(&self) -> &VersionReq {
        match self {
            ExtensionsConfig::Version(ref version) => version,
            ExtensionsConfig::Structured(config) => &config.version,
        }
    }

    pub fn networking(&self) -> bool {
        match self {
            ExtensionsConfig::Version(_) => false,
            ExtensionsConfig::Structured(config) => config.networking,
        }
    }

    pub fn stdout(&self) -> bool {
        match self {
            ExtensionsConfig::Version(_) => false,
            ExtensionsConfig::Structured(config) => config.stdout,
        }
    }

    pub fn stderr(&self) -> bool {
        match self {
            ExtensionsConfig::Version(_) => false,
            ExtensionsConfig::Structured(config) => config.stderr,
        }
    }

    pub fn environment_variables(&self) -> bool {
        match self {
            ExtensionsConfig::Version(_) => false,
            ExtensionsConfig::Structured(config) => config.environment_variables,
        }
    }

    pub fn max_pool_size(&self) -> Option<usize> {
        match self {
            ExtensionsConfig::Version(_) => None,
            ExtensionsConfig::Structured(config) => config.max_pool_size,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            ExtensionsConfig::Version(_) => None,
            ExtensionsConfig::Structured(config) => config.path.as_deref(),
        }
    }
}
