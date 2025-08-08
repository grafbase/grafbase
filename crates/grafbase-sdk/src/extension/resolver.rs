use crate::{
    component::AnyExtension,
    types::{
        Configuration, Error, IndexedSchema, OperationContext, ResolvedField, Response, SubgraphHeaders,
        SubgraphSchema, SubscriptionItem, Variables,
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
///     types::{Configuration, Error, ResolvedField, Response, SubgraphHeaders, SubgraphSchema, Variables},
/// };
///
/// #[derive(ResolverExtension)]
/// struct MyResolver {
///     config: Config
/// }
///
/// // Configuration in the TOML for this extension
/// #[derive(serde::Deserialize)]
/// struct Config {
///     #[serde(default)]
///     key: Option<String>
/// }
///
/// impl ResolverExtension for MyResolver {
///     fn new(subgraph_schemas: Vec<SubgraphSchema>, config: Configuration) -> Result<Self, Error> {
///         let config: Config = config.deserialize()?;
///         Ok(Self { config })
///     }
///
///     fn resolve(
///         &mut self,
///         prepared: &[u8],
///         headers: SubgraphHeaders,
///         variables: Variables,
///     ) -> Result<Response, Error> {
///         // field which must be resolved. The prepared bytes can be customized to store anything you need in the operation cache.
///         let field = ResolvedField::try_from(prepared)?;
///         Ok(Response::null())
///     }
/// }
/// ```
/// ## Configuration
///
/// The configuration provided in the `new` method is the one defined in the `grafbase.toml`
/// file by the extension user:
///
/// ```toml
/// [extensions.my-resolver.config]
/// key = "value"
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
/// [extensions.my-resolver]
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
/// method as a schema [crate::types::Directive]. The whole subgraph schema is also provided for
/// each subgraph where this extension is used. While the latter can be accessed with
/// [ResolvedField::directive()] in the [resolve()](ResolverExtension::resolve()) method.
///
pub trait ResolverExtension: Sized + 'static {
    /// Creates a new instance of the extension. The [Configuration] will contain all the
    /// configuration defined in the `grafbase.toml` by the extension user in a serialized format.
    /// Furthermore the complete subgraph schema is provided whenever this extension is used.
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
    /// # use grafbase_sdk::types::{Error, SubgraphSchema};
    /// # fn dummy(subgraph_schemas: Vec<SubgraphSchema>) -> Result<(), Error> {
    /// #[derive(serde::Deserialize)]
    /// struct HttpEndpoint {
    ///     name: String,
    ///     url: String
    /// }
    /// for schema in subgraph_schemas {
    ///     for directive in schema.directives() {
    ///         match directive.name() {
    ///             "httpEndpoint" => {
    ///                  let http_endpoint: HttpEndpoint = directive.arguments()?;
    ///             }
    ///             _ => unreachable!()
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn new(subgraph_schemas: Vec<SubgraphSchema>, config: Configuration) -> Result<Self, Error>;

    /// Prepares the field for resolution. The resulting byte array will be part of the operation
    /// cache. Backwards compatibility is not a concern as the cache is only re-used for the same
    /// schema and extension versions.
    /// By default the [ResolvedField] is cached for a simpler implementation.
    fn prepare(&mut self, field: ResolvedField<'_>) -> Result<Vec<u8>, Error> {
        Ok(field.into())
    }

    /// Resolves the field with the provided prepared bytes, headers and variables. With the
    /// default [prepare()](ResolverExtension::prepare()) you can retrieve all the relevant
    /// information with:
    /// ```rust
    /// # use grafbase_sdk::types::{SubgraphHeaders, Variables, Response, Error, ResolvedField};
    /// # fn resolve(prepared: &[u8], headers: SubgraphHeaders, variables: Variables) -> Result<Response, Error> {
    /// let field = ResolvedField::try_from(prepared)?;
    /// # Ok(Response::null())
    /// # }
    /// ```
    ///
    /// If you're not doing any data transformation it's best to forward the JSON or CBOR bytes,
    /// with [Response::json] and [Response::cbor] respectively, directly to the gateway. The
    /// gateway will always validate the subgraph data and deal with error propagation. Otherwise
    /// use [Response::data] to use the fastest supported serialization format.
    fn resolve(
        &mut self,
        ctx: &OperationContext,
        prepared: &[u8],
        headers: SubgraphHeaders,
        variables: Variables,
    ) -> Result<Response, Error>;

    /// Resolves a subscription for the given prepared bytes, headers, and variables.
    /// Subscriptions must implement the [Subscription] trait. It's also possible to provide a
    /// de-duplication key. If provided the gateway will first check if there is an existing
    /// subscription with the same key and if there is, re-use it for the new client. This greatly
    /// limits the impact on the upstream service. So you have two choices for the result type:
    /// - return a `Ok(subscription)` directly, without any de-duplication
    /// - return a `Ok((key, callback))` with an optional de-duplication key and a callback
    ///   function. The latter is only called if no existing subscription exists for the given key.
    #[allow(unused_variables)]
    fn resolve_subscription<'s>(
        &'s mut self,
        ctx: &'s OperationContext,
        prepared: &'s [u8],
        headers: SubgraphHeaders,
        variables: Variables,
    ) -> Result<impl IntoSubscription<'s>, Error> {
        unimplemented!("Subscription resolution not implemented for this resolver extension");

        // So that Rust doesn't complain about the unknown type
        #[allow(unused)]
        Ok(PhantomSubscription)
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
    /// - `Ok(Some(Data))` if a field output was available
    /// - `Ok(None)` if the subscription has ended normally
    /// - `Err(Error)` if an error occurred while retrieving the next field output
    fn next(&mut self) -> Result<Option<SubscriptionItem>, Error>;
}

pub type SubscriptionCallback<'s> = Box<dyn FnOnce() -> Result<Box<dyn Subscription + 's>, Error> + 's>;

pub trait IntoSubscription<'s> {
    fn into_deduplication_key_and_subscription_callback(self) -> (Option<Vec<u8>>, SubscriptionCallback<'s>);
}

impl<'s, S> IntoSubscription<'s> for S
where
    S: Subscription + 's,
{
    fn into_deduplication_key_and_subscription_callback(self) -> (Option<Vec<u8>>, SubscriptionCallback<'s>) {
        (None, Box::new(move || Ok(Box::new(self))))
    }
}

impl<'s, Callback, S> IntoSubscription<'s> for (Vec<u8>, Callback)
where
    Callback: FnOnce() -> Result<S, Error> + 's,
    S: Subscription + 's,
{
    fn into_deduplication_key_and_subscription_callback(self) -> (Option<Vec<u8>>, SubscriptionCallback<'s>) {
        (
            Some(self.0),
            Box::new(move || {
                let s = (self.1)()?;
                Ok(Box::new(s) as Box<dyn Subscription + 's>)
            }),
        )
    }
}

impl<'s, Callback, S> IntoSubscription<'s> for (Option<Vec<u8>>, Callback)
where
    Callback: FnOnce() -> Result<S, Error> + 's,
    S: Subscription + 's,
{
    fn into_deduplication_key_and_subscription_callback(self) -> (Option<Vec<u8>>, SubscriptionCallback<'s>) {
        (
            self.0,
            Box::new(move || {
                let s = (self.1)()?;
                Ok(Box::new(s) as Box<dyn Subscription + 's>)
            }),
        )
    }
}

impl<'s, Callback, S> IntoSubscription<'s> for (String, Callback)
where
    Callback: FnOnce() -> Result<S, Error> + 's,
    S: Subscription + 's,
{
    fn into_deduplication_key_and_subscription_callback(self) -> (Option<Vec<u8>>, SubscriptionCallback<'s>) {
        (
            Some(self.0.into()),
            Box::new(move || {
                let s = (self.1)()?;
                Ok(Box::new(s) as Box<dyn Subscription + 's>)
            }),
        )
    }
}

impl<'s, Callback, S> IntoSubscription<'s> for (Option<String>, Callback)
where
    Callback: FnOnce() -> Result<S, Error> + 's,
    S: Subscription + 's,
{
    fn into_deduplication_key_and_subscription_callback(self) -> (Option<Vec<u8>>, SubscriptionCallback<'s>) {
        (
            self.0.map(Into::into),
            Box::new(move || {
                let s = (self.1)()?;
                Ok(Box::new(s) as Box<dyn Subscription + 's>)
            }),
        )
    }
}

impl<'s, Callback, S> IntoSubscription<'s> for (Callback, Vec<u8>)
where
    Callback: FnOnce() -> Result<S, Error> + 's,
    S: Subscription + 's,
{
    fn into_deduplication_key_and_subscription_callback(self) -> (Option<Vec<u8>>, SubscriptionCallback<'s>) {
        (self.1, self.0).into_deduplication_key_and_subscription_callback()
    }
}

impl<'s, Callback, S> IntoSubscription<'s> for (Callback, Option<Vec<u8>>)
where
    Callback: FnOnce() -> Result<S, Error> + 's,
    S: Subscription + 's,
{
    fn into_deduplication_key_and_subscription_callback(self) -> (Option<Vec<u8>>, SubscriptionCallback<'s>) {
        (self.1, self.0).into_deduplication_key_and_subscription_callback()
    }
}

impl<'s, Callback, S> IntoSubscription<'s> for (Callback, String)
where
    Callback: FnOnce() -> Result<S, Error> + 's,
    S: Subscription + 's,
{
    fn into_deduplication_key_and_subscription_callback(self) -> (Option<Vec<u8>>, SubscriptionCallback<'s>) {
        (self.1, self.0).into_deduplication_key_and_subscription_callback()
    }
}

impl<'s, Callback, S> IntoSubscription<'s> for (Callback, Option<String>)
where
    Callback: FnOnce() -> Result<S, Error> + 's,
    S: Subscription + 's,
{
    fn into_deduplication_key_and_subscription_callback(self) -> (Option<Vec<u8>>, SubscriptionCallback<'s>) {
        (self.1, self.0).into_deduplication_key_and_subscription_callback()
    }
}

#[doc(hidden)]
pub fn register<T: ResolverExtension>() {
    pub(super) struct Proxy<T: ResolverExtension>(T);

    impl<T: ResolverExtension> AnyExtension for Proxy<T> {
        fn prepare(&mut self, field: ResolvedField<'_>) -> Result<Vec<u8>, Error> {
            self.0.prepare(field)
        }

        fn resolve(&mut self, prepared: &[u8], headers: SubgraphHeaders, variables: Variables) -> Response {
            self.0.resolve(&OperationContext, prepared, headers, variables).into()
        }

        fn resolve_subscription<'a>(
            &'a mut self,
            prepared: &'a [u8],
            headers: SubgraphHeaders,
            variables: Variables,
        ) -> Result<(Option<Vec<u8>>, SubscriptionCallback<'a>), Error> {
            let (key, callback) = self
                .0
                .resolve_subscription(&OperationContext, prepared, headers, variables)?
                .into_deduplication_key_and_subscription_callback();
            Ok((key, callback))
        }
    }

    crate::component::register_extension(Box::new(|subgraph_schemas, config| {
        let schemas = subgraph_schemas
            .into_iter()
            .map(IndexedSchema::from)
            .collect::<Vec<_>>();
        <T as ResolverExtension>::new(schemas.into_iter().map(SubgraphSchema).collect(), config)
            .map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}

struct PhantomSubscription;

impl Subscription for PhantomSubscription {
    fn next(&mut self) -> Result<Option<SubscriptionItem>, Error> {
        Ok(None) // No-op implementation for the phantom subscription
    }
}
