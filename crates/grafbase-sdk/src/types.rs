//! Type definitions of the input and output data structures of the SDK.

use std::time::Duration;

pub use http::StatusCode;
pub use minicbor_serde::error::DecodeError;
pub use serde::Deserialize;
use serde::{de::DeserializeOwned, Serialize};

use crate::wit;

/// The directive and its arguments which define the extension in the GraphQL SDK.
pub struct Directive(crate::wit::Directive);

impl Directive {
    /// The name of the directive.
    pub fn name(&self) -> &str {
        &self.0.name
    }

    /// The name of the subgraph this directive is part of.
    pub fn subgraph_name(&self) -> &str {
        &self.0.subgraph_name
    }

    /// The directive arguments. The output is a Serde structure, that must map to
    /// the arguments of the directive.
    ///
    /// Error is returned if the directive argument does not match the output structure.
    pub fn arguments<'de, T>(&'de self) -> Result<T, DecodeError>
    where
        T: Deserialize<'de>,
    {
        minicbor_serde::from_slice(&self.0.arguments)
    }
}

impl From<crate::wit::Directive> for Directive {
    fn from(value: crate::wit::Directive) -> Self {
        Self(value)
    }
}

/// The input data structure of the field.
pub struct FieldDefinition(crate::wit::FieldDefinition);

impl FieldDefinition {
    /// The name of the field.
    pub fn name(&self) -> &str {
        self.0.name.as_str()
    }

    /// The name of the field type.
    pub fn type_name(&self) -> &str {
        self.0.type_name.as_str()
    }
}

impl From<crate::wit::FieldDefinition> for FieldDefinition {
    fn from(value: crate::wit::FieldDefinition) -> Self {
        Self(value)
    }
}

/// Output responses from the field resolver.
pub struct FieldOutput(crate::wit::FieldOutput);

impl Default for FieldOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldOutput {
    /// Construct a new output response.
    pub fn new() -> Self {
        Self(crate::wit::FieldOutput { outputs: Vec::new() })
    }

    /// Constructs a new, empty output with at least the specified capacity.
    ///
    /// The output will be able to hold at least `capacity` elements without
    /// reallocating.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(crate::wit::FieldOutput {
            outputs: Vec::with_capacity(capacity),
        })
    }

    /// Push a new output data to the response.
    pub fn push_value<T>(&mut self, output: T)
    where
        T: Serialize,
    {
        let output =
            minicbor_serde::to_vec(output).expect("serialization error is Infallible, so it should never happen");

        self.0.outputs.push(Ok(output))
    }

    /// Push a new error to the response.
    pub fn push_error(&mut self, error: crate::wit::Error) {
        self.0.outputs.push(Err(error))
    }
}

impl From<FieldOutput> for crate::wit::FieldOutput {
    fn from(value: FieldOutput) -> Self {
        value.0
    }
}

/// A container for field inputs.
pub struct FieldInputs(Vec<Vec<u8>>);

impl FieldInputs {
    pub(crate) fn new(inputs: Vec<Vec<u8>>) -> Self {
        Self(inputs)
    }

    /// Deserializes each byte slice in the `FieldInputs` to a collection of items.
    pub fn deserialize<'de, T>(&'de self) -> Result<Vec<T>, Box<dyn std::error::Error>>
    where
        T: Deserialize<'de>,
    {
        self.0
            .iter()
            .map(|input| minicbor_serde::from_slice(input).map_err(|e| Box::new(e) as Box<dyn std::error::Error>))
            .collect()
    }
}

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
    pub fn deserialize<'de, T>(&'de self) -> Result<T, Box<dyn std::error::Error>>
    where
        T: Deserialize<'de>,
    {
        minicbor_serde::from_slice(&self.0).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
    }
}

/// A cache implementation for storing data between requests.
pub struct Cache;

impl Cache {
    /// Retrieves a value from the cache by key, initializing it if not present.
    ///
    /// If the value exists in the cache, deserializes and returns it.
    /// If not found, calls the initialization function, caches the result, and returns it.
    ///
    /// # Arguments
    ///
    /// * `key` - The cache key to look up
    /// * `init` - Function to initialize the value if not found in cache
    ///
    /// # Errors
    ///
    /// Returns an error if serialization/deserialization fails or if the init function fails
    pub fn get<F, T>(key: &str, init: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Result<CachedItem<T>, Box<dyn std::error::Error>>,
        T: Serialize + DeserializeOwned,
    {
        let value = crate::wit::Cache::get(key);

        if let Some(value) = value {
            Ok(minicbor_serde::from_slice(&value)?)
        } else {
            let value = init()?;
            let serialized = minicbor_serde::to_vec(&value.value)?;

            crate::wit::Cache::set(key, &serialized, value.duration.map(|d| d.as_millis() as u64));

            Ok(value.value)
        }
    }
}

/// A value to be stored in the cache with an optional time-to-live duration.
pub struct CachedItem<T> {
    value: T,
    duration: Option<Duration>,
}

impl<T> CachedItem<T>
where
    T: Serialize,
{
    /// Creates a new cached item with the given value and optional TTL duration.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to cache
    /// * `duration` - Optional time-to-live duration after which the item expires
    pub fn new(value: T, duration: Option<Duration>) -> Self
    where
        T: Serialize,
    {
        Self { value, duration }
    }
}

/// A structure representing an authentication token claims.
pub struct Token {
    claims: Vec<u8>,
}

impl From<Token> for wit::Token {
    fn from(token: Token) -> wit::Token {
        wit::Token { claims: token.claims }
    }
}

impl Token {
    /// Creates a new `Token` with the given claims.
    pub fn new<T>(claims: T) -> Self
    where
        T: Serialize,
    {
        Self {
            claims: minicbor_serde::to_vec(&claims)
                .expect("serialization error is Infallible, so it should never happen"),
        }
    }
}

/// A response containing a status code and multiple errors.
pub struct ErrorResponse(crate::wit::ErrorResponse);

impl From<ErrorResponse> for crate::wit::ErrorResponse {
    fn from(resp: ErrorResponse) -> Self {
        resp.0
    }
}

impl ErrorResponse {
    /// Creates a new `ErrorResponse` with the given HTTP status code.
    pub fn new(status_code: http::StatusCode) -> Self {
        Self(crate::wit::ErrorResponse {
            status_code: status_code.as_u16(),
            errors: Vec::new(),
        })
    }

    /// Adds a new error to the response.
    pub fn push_error(&mut self, error: crate::wit::Error) {
        self.0.errors.push(error);
    }
}
