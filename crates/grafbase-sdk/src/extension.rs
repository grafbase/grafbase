#![allow(static_mut_refs)]

pub mod authentication;
pub mod resolver;

pub use authentication::Authenticator;
pub use resolver::Resolver;

use crate::{
    types::{Configuration, FieldInputs},
    wit::{Directive, Error, ExtensionType, FieldDefinition, FieldOutput, Guest, Headers, SharedContext, Token},
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
    fn new(
        schema_directives: Vec<crate::types::Directive>,
        config: Configuration,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}

impl Guest for Component {
    fn init_gateway_extension(
        r#type: ExtensionType,
        directives: Vec<Directive>,
        configuration: Vec<u8>,
    ) -> Result<(), String> {
        let directives = directives.into_iter().map(Into::into).collect();
        let config = Configuration::new(configuration);

        let result = match r#type {
            ExtensionType::Resolver => resolver::init(directives, config),
            ExtensionType::Authentication => authentication::init(directives, config),
        };

        result.map_err(|e| e.to_string())
    }

    fn resolve_field(
        context: SharedContext,
        directive: Directive,
        definition: FieldDefinition,
        inputs: Vec<Vec<u8>>,
    ) -> Result<FieldOutput, Error> {
        let result = resolver::get_extension()?.resolve_field(
            context,
            directive.into(),
            definition.into(),
            FieldInputs::new(inputs),
        );

        result.map(Into::into)
    }

    fn resolve_subscription(
        context: SharedContext,
        directive: Directive,
        definition: FieldDefinition,
    ) -> Result<(), Error> {
        let subscriber =
            resolver::get_extension()?.resolve_subscription(context, directive.into(), definition.into())?;

        resolver::set_subscriber(subscriber);

        Ok(())
    }

    fn resolve_next_subscription_item() -> Result<Option<FieldOutput>, Error> {
        Ok(resolver::get_subscriber()?.next()?.map(Into::into))
    }

    fn authenticate(headers: Headers) -> Result<Token, crate::wit::ErrorResponse> {
        let result = authentication::get_extension()
            .map_err(|_| crate::wit::ErrorResponse {
                status_code: 500,
                errors: vec![Error {
                    extensions: Vec::new(),
                    message: String::from("internal server error"),
                }],
            })?
            .authenticate(headers);

        result.map(Into::into).map_err(Into::into)
    }
}
