use serde::{Deserialize, de::DeserializeOwned};
use wasmtime::component::{ComponentType, Lift, Lower};

use crate::{Error, GuestError};

/// Defines the type of the extension.
#[derive(Debug, Clone, Copy, Lower, ComponentType)]
#[component(enum)]
#[repr(u8)]
pub enum ExtensionType {
    /// A resolver extension can call the `resolve-field` function.
    #[component(name = "resolver")]
    Resolver,
    /// A resolver extension can call the `authenticate` function.
    #[component(name = "authentication")]
    Authentication,
}

/// A directive related to the extension.
#[derive(Debug, Clone, Lower, ComponentType)]
#[component(record)]
pub struct Directive {
    #[component(name = "name")]
    name: String,
    #[component(name = "subgraph-name")]
    subgraph_name: String,
    #[component(name = "arguments")]
    arguments: Vec<u8>,
}

impl Directive {
    /// Creates a new directive with the specified name and arguments.
    pub fn new(name: String, subgraph_name: String, arguments: impl serde::Serialize) -> Self {
        Self {
            name,
            subgraph_name,
            arguments: minicbor_serde::to_vec(arguments).unwrap(),
        }
    }
}

/// A definition of a field with a directive triggering the extension.
#[derive(Clone, Lower, ComponentType)]
#[component(record)]
pub struct FieldDefinition {
    /// The name of the field's type.
    #[component(name = "type-name")]
    pub type_name: String,
    /// The name of the field.
    #[component(name = "name")]
    pub name: String,
}

/// The output of a field resolver extension.
#[derive(Clone, Lift, ComponentType)]
#[component(record)]
pub struct FieldOutput {
    /// The raw bytes of the outputs in CBOR format.
    #[component(name = "outputs")]
    pub outputs: Vec<Result<Vec<u8>, GuestError>>,
}

impl FieldOutput {
    /// The outputs of the field resolver extension.
    pub fn serialize_outputs<S>(self) -> Vec<Result<S, Error>>
    where
        S: for<'a> Deserialize<'a>,
    {
        self.outputs
            .into_iter()
            .map(|result| match result {
                Ok(ref data) => minicbor_serde::from_slice(data).map_err(|e| Error::Internal(e.into())),
                Err(error) => Err(Error::Guest(error)),
            })
            .collect()
    }
}

#[derive(Clone, Lift, ComponentType)]
#[component(record)]
pub struct Token {
    #[component(name = "raw")]
    raw: Vec<u8>,
}

impl Token {
    pub fn deserialize<S>(&self) -> anyhow::Result<S>
    where
        S: DeserializeOwned,
    {
        Ok(minicbor_serde::from_slice(&self.raw)?)
    }
}
