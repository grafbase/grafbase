//! Type definitions of the input and output data structures of the SDK.

mod authorization;
mod directive;
mod directive_site;
mod elements;
mod error;
mod error_response;

pub use authorization::*;
pub use directive::*;
pub use directive_site::*;
pub use elements::*;
pub use error::*;
pub use error_response::*;

pub use http::StatusCode;
pub use serde::Deserialize;
use serde::Serialize;

use crate::wit;

/// Output responses from the field resolver.
pub struct FieldOutput(wit::FieldOutput);

impl Default for FieldOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldOutput {
    /// Construct a new output response.
    pub fn new() -> Self {
        Self(wit::FieldOutput { outputs: Vec::new() })
    }

    /// Constructs a new, empty output with at least the specified capacity.
    ///
    /// The output will be able to hold at least `capacity` elements without
    /// reallocating.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(wit::FieldOutput {
            outputs: Vec::with_capacity(capacity),
        })
    }

    /// Push a new output data to the response.
    pub fn push_value<T>(&mut self, output: T)
    where
        T: Serialize,
    {
        let output = crate::cbor::to_vec(output).expect("serialization error is Infallible, so it should never happen");

        self.0.outputs.push(Ok(output));
    }

    /// Push a new error to the response.
    pub fn push_error(&mut self, error: impl Into<Error>) {
        self.0.outputs.push(Err(Into::<Error>::into(error).into()));
    }
}

impl From<FieldOutput> for wit::FieldOutput {
    fn from(value: FieldOutput) -> Self {
        value.0
    }
}

/// A container for field inputs.
#[derive(Debug)]
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

/// A structure representing an authentication token claims.
pub struct Token {
    claims: Vec<u8>,
}

impl From<Token> for wit::Token {
    fn from(token: Token) -> wit::Token {
        wit::Token { raw: token.claims }
    }
}

impl Token {
    /// Creates a new `Token` with the given claims. The claims can be defined by the extension,
    /// but to use the [`@requiredScopes`](https://grafbase.com/docs/reference/graphql-directives#requiresscopes)
    /// directive, the claims must contain a `scopes` field with an array of strings.
    pub fn new<T>(claims: T) -> Self
    where
        T: serde::Serialize,
    {
        Self {
            claims: crate::cbor::to_vec(&claims).expect("serialization error is Infallible, so it should never happen"),
        }
    }
}
