#![allow(static_mut_refs)]

pub mod resolver;

pub use resolver::Resolver;

use crate::{
    wit::{Directive, Error, ExtensionType, FieldInput, FieldOutput, Guest, SharedContext},
    Component,
};

/// A trait representing an extension that can be initialized from schema directives.
///
/// This trait is intended to define a common interface for extensions in Grafbase Gateway,
/// particularly focusing on their initialization. Extensions are constructed using
/// a vector of `Directive` instances provided by the type definitions in the schema.
pub trait Extension {
    /// Creates a new instance of the extension from the given schema directives.
    ///
    /// The directives must be defined in the extension configuration, and written
    /// to the federated schema. The directives are deserialized from their GraphQL
    /// definitions to the corresponding `Directive` instances.
    fn new(schema_directives: Vec<crate::types::Directive>) -> Self
    where
        Self: Sized;
}

impl Guest for Component {
    fn init_gateway_extension(r#type: ExtensionType, directives: Vec<Directive>) -> Result<(), String> {
        match r#type {
            ExtensionType::Resolver => resolver::init(directives.into_iter().map(Into::into).collect()),
        }
    }

    fn resolve_field(
        context: SharedContext,
        directive: Directive,
        inputs: Vec<FieldInput>,
    ) -> Result<FieldOutput, Error> {
        let result = resolver::get_extension()?.resolve_field(
            context,
            directive.into(),
            inputs.into_iter().map(Into::into).collect(),
        );

        result.map(Into::into)
    }
}
