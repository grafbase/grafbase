use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Deserializer};

pub(super) const GATEWAY_BINARY_NAME: &str = "grafbase-gateway";
pub(super) const CLI_BINARY_NAME: &str = "grafbase";

/// Log level for the test process output. Default value is `LogLevel::Error`.
#[derive(Debug, Clone, Default)]
pub enum LogLevel {
    /// Show all output from traces upwards.
    Trace,
    /// Show all output from debug upwards.
    Debug,
    /// Show all output from info upwards.
    #[default]
    Info,
    /// Show all output from warn upwards.
    Warn,
    /// Show only error messages.
    Error,
    /// Show all output from engine, debug upwards.
    EngineDebug,
    /// Extra verbose logs, show all output from engine, trace upwards.
    EngineTrace,
    /// Extra extra verbose logs
    WasiDebug,
    /// You know what you need
    Custom(String),
}

impl AsRef<str> for LogLevel {
    fn as_ref(&self) -> &str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            LogLevel::EngineDebug => "engine=debug",
            LogLevel::EngineTrace => "engine=trace",
            LogLevel::WasiDebug => "wasi_component_loader=debug",
            LogLevel::Custom(level) => level.as_str(),
        }
    }
}

impl From<String> for LogLevel {
    fn from(level: String) -> Self {
        LogLevel::Custom(level)
    }
}

impl From<&str> for LogLevel {
    fn from(level: &str) -> Self {
        level.to_string().into()
    }
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct ExtensionToml {
    pub extension: ExtensionDefinition,
}

#[derive(Debug, serde::Deserialize)]
pub(super) struct ExtensionDefinition {
    pub name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct GatewayToml {
    #[serde(default)]
    pub extensions: HashMap<String, ExtensionConfig>,
    #[serde(default)]
    pub wasm: WasmConfig,
    #[serde(flatten)]
    pub rest: toml::Table,
}

#[derive(Default, Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WasmConfig {
    pub cache_path: Option<PathBuf>,
}

#[derive(Debug)]
pub(super) enum ExtensionConfig {
    Version(String),
    Structured(StructuredExtensionConfig),
}

impl<'de> Deserialize<'de> for ExtensionConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{Error, MapAccess, Visitor, value::MapAccessDeserializer};
        struct ExtensionConfigVisitor;

        impl<'de> Visitor<'de> for ExtensionConfigVisitor {
            type Value = ExtensionConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a version or a config map")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                value.parse().map(ExtensionConfig::Version).map_err(Error::custom)
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                StructuredExtensionConfig::deserialize(MapAccessDeserializer::new(&mut map))
                    .map(ExtensionConfig::Structured)
            }
        }

        deserializer.deserialize_any(ExtensionConfigVisitor)
    }
}

impl serde::Serialize for ExtensionConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ExtensionConfig::Version(version) => serializer.serialize_str(version),
            ExtensionConfig::Structured(config) => config.serialize(serializer),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct StructuredExtensionConfig {
    pub version: Option<String>,
    pub path: Option<String>,
    #[serde(flatten)]
    pub rest: toml::Table,
}
