use crate::{
    component::AnyExtension,
    host::Headers,
    types::{Configuration, Error, FieldDefinitionDirective, FieldInputs, FieldOutput, SchemaDirective},
};

/// A trait that extends `Extension` and provides functionality for resolving fields.
///
/// Implementors of this trait are expected to provide a method to resolve field values based on
/// the given context, directive, and inputs. This is typically used in scenarios where field
/// resolution logic needs to be encapsulated within a resolver object, allowing for modular
/// and reusable code design.
pub trait ResolverExtension: Sized + 'static {
    /// Creates a new instance of the extension.
    ///
    /// # Arguments
    ///
    /// * `schema_directives` - List of all schema directives for all subgraphs defined in this
    ///                         extension.
    /// * `config` - The configuration for this extension, from the gateway TOML.
    ///
    /// # Returns
    ///
    /// Returns an instance of this resolver. Upon failure, every call to this extension will fail.
    /// Similar to how every request to a subgraph would fail if it went down.
    fn new(schema_directives: Vec<SchemaDirective>, config: Configuration) -> Result<Self, Error>;

    /// Resolves a field value based on the given context, directive, definition, and inputs.
    ///
    /// # Arguments
    ///
    /// * `headers` - The subgraph headers associated with this field resolution
    /// * `subgraph_name` - The name of the subgraph associated with this field resolution
    /// * `directive` - The directive associated with this field resolution
    /// * `definition` - The field definition containing metadata
    /// * `inputs` - The input values provided for this field
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either the resolved `FieldOutput` value or an `Error`
    fn resolve_field(
        &mut self,
        headers: Headers,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
        inputs: FieldInputs,
    ) -> Result<FieldOutput, Error>;

    /// Resolves a subscription field by setting up a subscription handler.
    ///
    /// # Arguments
    ///
    /// * `headers` - The subgraph headers associated with this field resolution
    /// * `directive` - The directive associated with this subscription field
    /// * `definition` - The field definition containing metadata about the subscription
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing either a boxed `Subscriber` implementation or an `Error`
    fn resolve_subscription(
        &mut self,
        headers: Headers,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
    ) -> Result<Box<dyn Subscription>, Error>;

    /// Returns a key for a subscription field.
    ///
    /// This method is used to identify unique subscription channels or connections
    /// when managing multiple active subscriptions. The returned key can be
    /// used to track, manage, or deduplicate subscriptions.
    ///
    /// # Arguments
    ///
    /// * `headers` - The subgraph headers associated with this subscription
    /// * `subgraph_name` - The name of the subgraph associated with this subscription
    /// * `directive` - The directive associated with this subscription field
    ///
    /// # Returns
    ///
    /// Returns an `Option<Vec<u8>>` containing either a unique key for this
    /// subscription or `None` if no deduplication is desired.
    #[allow(unused)]
    fn subscription_key(
        &mut self,
        headers: Headers,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
    ) -> Option<Vec<u8>> {
        None
    }
}

/// A trait for consuming field outputs from streams.
///
/// This trait provides an abstraction over different implementations
/// of subscriptions to field output streams. Implementors should handle
/// the details of their specific transport mechanism while providing a
/// consistent interface for consumers.
pub trait Subscription {
    /// Retrieves the next field output from the subscription.
    ///
    /// Returns:
    /// - `Ok(Some(FieldOutput))` if a field output was available
    /// - `Ok(None)` if the subscription has ended normally
    /// - `Err(Error)` if an error occurred while retrieving the next field output
    fn next(&mut self) -> Result<Option<FieldOutput>, Error>;
}

#[doc(hidden)]
pub fn register<T: ResolverExtension>() {
    pub(super) struct Proxy<T: ResolverExtension>(T);

    impl<T: ResolverExtension> AnyExtension for Proxy<T> {
        fn resolve_field(
            &mut self,
            headers: Headers,
            subgraph_name: &str,
            directive: FieldDefinitionDirective<'_>,
            inputs: FieldInputs,
        ) -> Result<FieldOutput, Error> {
            ResolverExtension::resolve_field(&mut self.0, headers, subgraph_name, directive, inputs)
        }
        fn resolve_subscription(
            &mut self,
            headers: Headers,
            subgraph_name: &str,
            directive: FieldDefinitionDirective<'_>,
        ) -> Result<Box<dyn Subscription>, Error> {
            ResolverExtension::resolve_subscription(&mut self.0, headers, subgraph_name, directive)
        }

        fn subscription_key(
            &mut self,
            headers: Headers,
            subgraph_name: &str,
            directive: FieldDefinitionDirective<'_>,
        ) -> Result<Option<Vec<u8>>, Error> {
            Ok(ResolverExtension::subscription_key(
                &mut self.0,
                headers,
                subgraph_name,
                directive,
            ))
        }
    }

    crate::component::register_extension(Box::new(|schema_directives, config| {
        <T as ResolverExtension>::new(schema_directives, config)
            .map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
