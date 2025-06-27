use serde::Deserialize;

use crate::SdkError;

/// Configuration data for the extension, from the gateway toml config.
pub struct Configuration(Vec<u8>);

impl Configuration {
    /// Creates a new `Configuration` from a CBOR byte vector.
    pub(crate) fn new(config: Vec<u8>) -> Self {
        Self(config)
    }

    /// Deserializes the configuration bytes into the requested type.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn deserialize<'de, T>(&'de self) -> Result<T, SdkError>
    where
        T: Deserialize<'de>,
    {
        let mut deserializer = minicbor_serde::Deserializer::new(&self.0);
        serde_path_to_error::deserialize(&mut deserializer).map_err(|err| {
            if err.path().iter().len() == 0
                || err
                    .path()
                    .iter()
                    .all(|seg| matches!(seg, serde_path_to_error::Segment::Unknown))
            {
                let inner_err = err.into_inner();
                format!("Failed to deserialize configuration: {inner_err}").into()
            } else {
                format!("Failed to deserialize configuration at {err}").into()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestConfig<S> {
        name: String,
        settings: S,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Settings {
        enabled: bool,
        count: u32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct InvalidSettings {
        enabled: String,
        count: u32,
    }

    #[test]
    fn test_deserialize() {
        // Test successful deserialization
        let config_bytes = crate::cbor::to_vec(TestConfig {
            name: "test".to_string(),
            settings: Settings {
                enabled: true,
                count: 42,
            },
        })
        .unwrap();
        let configuration = Configuration::new(config_bytes);
        let result: TestConfig<Settings> = configuration.deserialize().unwrap();
        assert_eq!(result.name, "test");
        assert!(result.settings.enabled);
        assert_eq!(result.settings.count, 42);

        // Test error with path reporting
        let invalid_bytes = crate::cbor::to_vec(&TestConfig {
            name: "test".to_string(),
            settings: InvalidSettings {
                enabled: "not_a_bool".to_string(),
                count: 42,
            },
        })
        .unwrap();
        let invalid_configuration = Configuration::new(invalid_bytes);
        let err = invalid_configuration.deserialize::<TestConfig<Settings>>().unwrap_err();

        insta::assert_snapshot!(err, @"Failed to deserialize configuration at settings.enabled: unexpected type string at position 29: expected bool");

        // Test error with absent config
        let invalid_bytes = crate::cbor::to_vec(serde_json::Value::Null).unwrap();
        let invalid_configuration = Configuration::new(invalid_bytes);
        let err = invalid_configuration.deserialize::<TestConfig<Settings>>().unwrap_err();

        insta::assert_snapshot!(err, @"Failed to deserialize configuration: unexpected type null at position 0: expected map");
    }
}
