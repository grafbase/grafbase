use serde::Deserialize;

use crate::{cbor, SdkError};

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
        cbor::from_slice(&self.0).map_err(Into::into)
    }
}
