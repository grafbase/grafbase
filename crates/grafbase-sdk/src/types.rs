//! Type definitions of the input and output data structures of the SDK.

pub use minicbor_serde::error::DecodeError;
pub use serde::Deserialize;
use serde::Serialize;

/// The directive and its arguments which define the extension in the GraphQL SDK.
pub struct Directive(crate::wit::Directive);

impl Directive {
    /// The name of the directive.
    pub fn name(&self) -> &str {
        &self.0.name
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
