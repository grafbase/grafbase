use std::path::{Path, PathBuf};

use semver::VersionReq;
use serde::{Deserialize, Deserializer};

#[derive(PartialEq, Debug, Clone)]
pub enum ExtensionConfig {
    Version(VersionReq),
    Structured(StructuredExtensionConfig),
}

// #[serde(untagged)] results is very poor errors as it tries to deserialize the variants one by
// one, ignoring the errors and ending with: `data did not match any variant of untagged enum ExtensionConfig`.
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

#[derive(PartialEq, serde::Deserialize, Debug, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct StructuredExtensionConfig {
    pub version: VersionReq,
    pub path: Option<PathBuf>,
    pub networking: Option<bool>,
    pub stdout: Option<bool>,
    pub stderr: Option<bool>,
    pub environment_variables: Option<bool>,
    pub max_pool_size: Option<usize>,
    pub config: Option<toml::Value>,
}

impl Default for StructuredExtensionConfig {
    fn default() -> Self {
        Self {
            version: VersionReq::parse("*").unwrap(),
            path: None,
            networking: None,
            stdout: None,
            stderr: None,
            environment_variables: None,
            max_pool_size: None,
            config: None,
        }
    }
}

impl ExtensionConfig {
    pub fn version(&self) -> &VersionReq {
        match self {
            ExtensionConfig::Version(version) => version,
            ExtensionConfig::Structured(config) => &config.version,
        }
    }

    pub fn networking(&self) -> Option<bool> {
        match self {
            ExtensionConfig::Version(_) => None,
            ExtensionConfig::Structured(config) => config.networking,
        }
    }

    pub fn stdout(&self) -> Option<bool> {
        match self {
            ExtensionConfig::Version(_) => None,
            ExtensionConfig::Structured(config) => config.stdout,
        }
    }

    pub fn stderr(&self) -> Option<bool> {
        match self {
            ExtensionConfig::Version(_) => None,
            ExtensionConfig::Structured(config) => config.stderr,
        }
    }

    pub fn environment_variables(&self) -> Option<bool> {
        match self {
            ExtensionConfig::Version(_) => None,
            ExtensionConfig::Structured(config) => config.environment_variables,
        }
    }

    pub fn max_pool_size(&self) -> Option<usize> {
        match self {
            ExtensionConfig::Version(_) => None,
            ExtensionConfig::Structured(config) => config.max_pool_size,
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            ExtensionConfig::Version(_) => None,
            ExtensionConfig::Structured(config) => config.path.as_deref(),
        }
    }

    pub fn config(&self) -> Option<&toml::Value> {
        match self {
            ExtensionConfig::Version(_) => None,
            ExtensionConfig::Structured(config) => config.config.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_string_expansion_in_extension_config_missing_env_var() {
        let toml = r#"
            [extensions.dummy]
            version = "1.0"

            [extensions.dummy.config.test]
            key = "value"
            key_from_env = "{{ env.test }}"
        "#;

        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.toml");
        std::fs::write(&path, toml).unwrap();
        let err = crate::Config::load(&path).unwrap_err().to_string();

        insta::assert_snapshot!(err, @"At extensions.dummy.config.test.key_from_env, failed substituing environment variable: environment variable not found: `test`");
    }

    #[test]
    fn dynamic_string_expansion_in_extension_config_no_env_var() {
        let toml = r#"
            version = "1.0"

            [config.test]
            key = "value"
            other_key = "abcd"
        "#;

        toml::from_str::<StructuredExtensionConfig>(toml).unwrap();
    }
}
