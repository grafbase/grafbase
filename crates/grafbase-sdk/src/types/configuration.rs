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
                err.into_inner().into()
            } else {
                format!("Failed to deserialize configuration at {}", err).into()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestConfig {
        name: String,
        settings: Settings,
    }

    #[derive(Debug, Deserialize)]
    struct Settings {
        enabled: bool,
        count: u32,
    }

    #[test]
    fn test_deserialize() {
        // Test successful deserialization
        let config = serde_json::json!({
            "name": "test",
            "settings": {
                "enabled": true,
                "count": 42
            }
        });
        let config_bytes = crate::cbor::to_vec(&config).unwrap();
        let configuration = Configuration::new(config_bytes);
        let result: TestConfig = configuration.deserialize().unwrap();
        assert_eq!(result.name, "test");
        assert!(result.settings.enabled);
        assert_eq!(result.settings.count, 42);

        // Test error with path reporting
        let invalid_config = serde_json::json!({
            "name": "test",
            "settings": {
                "enabled": "not_a_bool", // This should be a boolean
                "count": 42
            }
        });
        let invalid_bytes = crate::cbor::to_vec(&invalid_config).unwrap();
        let invalid_configuration = Configuration::new(invalid_bytes);
        let err = invalid_configuration.deserialize::<TestConfig>().unwrap_err();

        insta::assert_snapshot!(err, @"Failed to deserialize configuration at settings.enabled: unexpected type string at position 37: expected bool");
    }
}
