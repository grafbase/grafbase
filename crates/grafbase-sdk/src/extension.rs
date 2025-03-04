pub mod authentication;
pub mod authorization;
pub mod resolver;

pub use authentication::Authenticator;
pub use authorization::Authorizer;
pub use resolver::Resolver;

use crate::types::Configuration;

/// A trait representing an extension that can be initialized from schema directives.
///
/// This trait is intended to define a common interface for extensions in Grafbase Gateway,
/// particularly focusing on their initialization. Extensions are constructed using
/// a vector of `Directive` instances provided by the type definitions in the schema.
pub trait Extension: 'static {
    /// Creates a new instance of the extension from the given schema directives.
    ///
    /// The directives must be defined in the extension configuration, and written
    /// to the federated schema. The directives are deserialized from their GraphQL
    /// definitions to the corresponding `Directive` instances.
    fn new(
        schema_directives: Vec<crate::types::SchemaDirective>,
        config: Configuration,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        Self: Sized;
}
