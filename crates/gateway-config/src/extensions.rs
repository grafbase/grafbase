use std::{
    path::{Path, PathBuf},
    str::FromStr as _,
};

use semver::VersionReq;
use serde::Deserialize as _;

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
    pub networking: Option<bool>,
    pub stdout: Option<bool>,
    pub stderr: Option<bool>,
    pub environment_variables: Option<bool>,
    pub max_pool_size: Option<usize>,
    #[serde(deserialize_with = "deserialize_extension_custom_config")]
    pub config: Option<toml::Value>,
}

fn deserialize_extension_custom_config<'de, D>(deserializer: D) -> Result<Option<toml::Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut value = Option::<toml::Value>::deserialize(deserializer)?;

    fn expand_dynamic_strings(value: &mut toml::Value) -> Result<(), String> {
        match value {
            toml::Value::String(s) => {
                let substituted = serde_dynamic_string::DynamicString::<String>::from_str(s)?;
                *s = substituted.into_inner();
            }
            toml::Value::Array(values) => {
                for value in values {
                    expand_dynamic_strings(value)?;
                }
            }
            toml::Value::Table(map) => {
                for (_, value) in map {
                    expand_dynamic_strings(value)?;
                }
            }
            toml::Value::Integer(_) | toml::Value::Float(_) | toml::Value::Boolean(_) | toml::Value::Datetime(_) => (),
        }

        Ok(())
    }

    if let Some(value) = &mut value {
        expand_dynamic_strings(value).map_err(|err| {
            serde::de::Error::custom(format!(
                "Error expanding dynamic strings in extension configuration: {err}"
            ))
        })?;
    }

    Ok(value)
}

impl Default for StructuredExtensionsConfig {
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
            ExtensionsConfig::Version(version) => version,
            ExtensionsConfig::Structured(config) => &config.version,
        }
    }

    pub fn networking(&self) -> Option<bool> {
        match self {
            ExtensionsConfig::Version(_) => None,
            ExtensionsConfig::Structured(config) => config.networking,
        }
    }

    pub fn stdout(&self) -> Option<bool> {
        match self {
            ExtensionsConfig::Version(_) => None,
            ExtensionsConfig::Structured(config) => config.stdout,
        }
    }

    pub fn stderr(&self) -> Option<bool> {
        match self {
            ExtensionsConfig::Version(_) => None,
            ExtensionsConfig::Structured(config) => config.stderr,
        }
    }

    pub fn environment_variables(&self) -> Option<bool> {
        match self {
            ExtensionsConfig::Version(_) => None,
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

    pub fn config(&self) -> Option<&toml::Value> {
        match self {
            ExtensionsConfig::Version(_) => None,
            ExtensionsConfig::Structured(config) => config.config.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_string_expansion_in_extension_config_missing_env_var() {
        let toml = r#"
            version = "1.0"

            [config.test]
            key = "value"
            key_from_env = "{{ env.test }}"
        "#;

        let err = toml::from_str::<StructuredExtensionsConfig>(toml)
            .unwrap_err()
            .to_string();

        insta::assert_snapshot!(err, @r#"
        TOML parse error at line 4, column 14
          |
        4 |             [config.test]
          |              ^^^^^^
        Error expanding dynamic strings in extension configuration: environment variable not found: `test`
        "#);
    }

    #[test]
    fn dynamic_string_expansion_in_extension_config_no_env_var() {
        let toml = r#"
            version = "1.0"

            [config.test]
            key = "value"
            other_key = "abcd"
        "#;

        toml::from_str::<StructuredExtensionsConfig>(toml).unwrap();
    }
}
