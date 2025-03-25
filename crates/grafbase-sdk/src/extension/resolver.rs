use crate::{
    component::AnyExtension,
    types::{
        Configuration, Error, FieldDefinitionDirective, FieldInputs, FieldOutputs, SchemaDirective, SubgraphHeaders,
        SubscriptionOutput,
    },
};

/// A resolver extension is called by the gateway to resolve a specific field.
///
/// # Example
///
/// You can initialize a new resolver extension with the Grafbase CLI:
///
/// ```bash
/// grafbase extension init --type resolver my-resolver
/// ```
///
/// This will generate the following:
///
/// ```rust
/// use grafbase_sdk::{
///     ResolverExtension,
///     types::{SubgraphHeaders, FieldDefinitionDirective, FieldInputs, FieldOutputs, Configuration, Error, SchemaDirective}
/// };
///
/// #[derive(ResolverExtension)]
/// struct MyResolver {
///   config: Config
/// }
///
/// #[derive(serde::Deserialize)]
/// struct Config {
///   my_custom_key: String
/// }
///
/// impl ResolverExtension for MyResolver {
///    fn new(schema_directives: Vec<SchemaDirective>, config: Configuration) -> Result<Self, Error> {
///        let config: Config = config.deserialize()?;
///        Ok(Self { config })
///    }
///
///    fn resolve_field(
///        &mut self,
///        headers: SubgraphHeaders,
///        subgraph_name: &str,
///        directive: FieldDefinitionDirective<'_>,
///        inputs: FieldInputs<'_>,
///    ) -> Result<FieldOutputs, Error> {
///         unimplemented!()
///    }
/// }
/// ```
/// ## Configuration
///
/// The configuration provided in the `new` method is the one defined in the `grafbase.toml`
/// file by the extension user:
///
/// ```toml
/// [extensions.my-auth.config]
/// my_custom_key = "value"
/// ```
///
/// Once your business logic is written down you can compile your extension with:
///
/// ```bash
/// grafbase extension build
/// ```
///
/// It will generate all the necessary files in a `build` directory which you can specify in the
/// `grafbase.toml` configuration with:
///
/// ```toml
/// [extensions.my-auth]
/// path = "<project path>/build"
/// ```
///
/// ## Directives
///
/// In addition to the Rust extension, a `definitions.graphql` file will be also generated. It
/// should define directives for subgraph owners and any necessary input types, scalars or enum
/// necessary for those. Directives have two purposes for resolvers: define which fields can be
/// resolved, providing the necessary metadata for it, and provide global metadata with schema
/// directive.
///
/// A HTTP resolver extension could have the following directives for example:
///
/// ```graphql
/// scalar URL
///
/// directive @httpEndpoint(name: String!, url: URL!) on SCHEMA
///
/// directive @http(endpoint: String!, path: String!) on FIELD_DEFINITION
/// ```
///
/// The `@httpEndpoint` directive would be provided during the [new()](ResolverExtension::new())
/// method as a [SchemaDirective]. While the latter would be provided as a [FieldDefinitionDirective] during
/// the [resolve_field()](ResolverExtension::resolve_field()) method.
///
#[allow(unused_variables)]
pub trait ResolverExtension: Sized + 'static {
    /// Creates a new instance of the extension. The [Configuration] will contain all the
    /// configuration defined in the `grafbase.toml` by the extension user in a serialized format.
    /// Furthermore all schema directives from all subgraphs will be provided as
    /// [SchemaDirective]s.
    ///
    /// # Configuration example
    ///
    /// The following TOML configuration:
    /// ```toml
    /// [extensions.my-auth.config]
    /// my_custom_key = "value"
    /// ```
    ///
    /// can be easily deserialized with:
    ///
    /// ```rust
    /// # use grafbase_sdk::types::{Configuration, Error};
    /// # fn dummy(config: Configuration) -> Result<(), Error> {
    /// #[derive(serde::Deserialize)]
    /// struct Config {
    ///     my_custom_key: String
    /// }
    ///
    /// let config: Config = config.deserialize()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Directive example
    ///
    /// ```graphql
    /// extend schema @httpEdnpoint(name: "example", url: "https://example.com")
    /// ```
    ///
    /// can be easily deserialized with:
    ///
    /// ```rust
    /// # use grafbase_sdk::types::{Error, SchemaDirective};
    /// # fn dummy(schema_directives: Vec<SchemaDirective>) -> Result<(), Error> {
    /// #[derive(serde::Deserialize)]
    /// struct HttpEndpoint {
    ///     name: String,
    ///     url: String
    /// }
    ///
    /// let config: Vec<HttpEndpoint> = schema_directives
    ///     .into_iter()
    ///     .map(|dir| dir.arguments())
    ///     .collect::<Result<_, _>>()?;
    /// # Ok(())
    /// # }
    /// ```
    fn new(schema_directives: Vec<SchemaDirective>, config: Configuration) -> Result<Self, Error>;

    /// Resolves a GraphQL field. This function receives a batch of inputs and is called at most once per
    /// query field.
    ///
    /// Supposing we have the following directive applied on this schema:
    ///
    /// ```graphql
    /// extend schema
    ///  @link(
    ///    url: "https://specs.grafbase.com/grafbase"
    ///    import: ["FieldSet"]
    ///  )
    ///
    /// directive @resolve(fields: FieldSet!) on FIELD_DEFINITION
    /// ```
    ///
    /// ```graphql
    /// type Query {
    ///    users: [User] # from a different subgraph
    /// }
    ///
    /// type User {
    ///     id : ID!
    ///     field: JSON @resolve(fields: "id")
    /// }
    /// ```
    ///
    /// and a query like:
    ///
    /// ```graphql
    /// query {
    ///    users {
    ///       field
    ///    }
    /// }
    /// ```
    ///
    /// The subgraph providing `Query.users` will return an arbitrary number N of users. Instead of
    /// being called N times, this resolver will be called once with a [FieldInputs] containing N
    /// [FieldInput](crate::types::FieldInput). This allows you to batch everything together at
    /// once.
    ///
    /// ```rust
    /// # use grafbase_sdk::types::{SubgraphHeaders, FieldDefinitionDirective, FieldInputs, FieldOutputs, Error};
    /// # fn resolve_field(
    /// #    headers: SubgraphHeaders,
    /// #    subgraph_name: &str,
    /// #    directive: FieldDefinitionDirective<'_>,
    /// #    inputs: FieldInputs<'_>,
    /// # ) -> Result<FieldOutputs, Error> {
    /// // Static arguments passed on to the directive that do not depend on the response data.
    /// #[derive(serde::Deserialize)]
    /// struct StaticArguments<'a> {
    ///     #[serde(borrow)]
    ///     endpoint_name: &'a str,
    /// }
    /// let StaticArguments { endpoint_name } = directive.arguments()?;
    ///
    /// let mut builder = FieldOutputs::builder(inputs);
    /// for input in inputs {
    ///     // Field arguments that depend on response data.
    ///     #[derive(serde::Deserialize)]
    ///     struct ResponseArguments<'a> {
    ///         #[serde(borrow)]
    ///         id: &'a str,
    ///     }
    ///
    ///     let ResponseArguments { id } = directive.arguments()?;
    ///     builder.insert(input, "data");
    /// }
    ///
    /// Ok(builder.build())
    /// # }
    /// ```
    ///
    /// [FieldOutputs] can also be initialized with a single error or a single data for convenience.
    ///
    /// We also want to support providing raw JSON and CBOR bytes directly for batched and
    /// non-batched data later on, if it's of interested let us know!
    ///
    /// In addition to this the method also receives the subgraph `headers` after all the
    /// subgraph-related header rules. And metadata the
    /// [FieldDefinitionDirectiveSite](crate::types::FieldDefinitionDirectiveSite) is also
    /// available with [directive.site()](crate::types::FieldDefinitionDirective::site()).
    fn resolve_field(
        &mut self,
        headers: SubgraphHeaders,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
        inputs: FieldInputs<'_>,
    ) -> Result<FieldOutputs, Error>;

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
        headers: SubgraphHeaders,
        subgraph_name: &str,
        directive: FieldDefinitionDirective<'_>,
    ) -> Result<Box<dyn Subscription>, Error> {
        unimplemented!()
    }

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
    fn subscription_key(
        &mut self,
        headers: &SubgraphHeaders,
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
    /// - `Ok(Some(FieldOutputs))` if a field output was available
    /// - `Ok(None)` if the subscription has ended normally
    /// - `Err(Error)` if an error occurred while retrieving the next field output
    fn next(&mut self) -> Result<Option<SubscriptionOutput>, Error>;
}

#[doc(hidden)]
pub fn register<T: ResolverExtension>() {
    pub(super) struct Proxy<T: ResolverExtension>(T);

    impl<T: ResolverExtension> AnyExtension for Proxy<T> {
        fn resolve_field(
            &mut self,
            headers: SubgraphHeaders,
            subgraph_name: &str,
            directive: FieldDefinitionDirective<'_>,
            inputs: FieldInputs<'_>,
        ) -> Result<FieldOutputs, Error> {
            ResolverExtension::resolve_field(&mut self.0, headers, subgraph_name, directive, inputs)
        }
        fn resolve_subscription(
            &mut self,
            headers: SubgraphHeaders,
            subgraph_name: &str,
            directive: FieldDefinitionDirective<'_>,
        ) -> Result<Box<dyn Subscription>, Error> {
            ResolverExtension::resolve_subscription(&mut self.0, headers, subgraph_name, directive)
        }

        fn subscription_key(
            &mut self,
            headers: &SubgraphHeaders,
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
