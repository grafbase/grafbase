use crate::{
    component::AnyExtension,
    types::{
        AuthenticatedRequestContext, AuthorizationDecisions, AuthorizeQueryOutput, AuthorizedOperationContext,
        Configuration, Error, ErrorResponse, Headers, QueryElements, ResponseElements, SubgraphHeaders,
    },
};

/// An authorization extension can grant or deny access to fields, objects, interfaces, unions,
/// scalars or enums. It's composed of two parts:
/// - a Wasm module holding the business logic executed by the gateway.
/// - GraphQL directives, provided to subgraph owners to annotate which elements need
///   authorization.
///
/// Authorization is done in two steps:
/// - Before starting the execution, [authorize_query()](AuthorizationExtension::authorize_query()) will be called once with all the elements
///   that must be authorized. Any denied elements will *not* be requested from subgraphs.
/// - Optionally, if authorization depends on response data, [authorize_response()](AuthorizationExtension::authorize_response()) will be called
///   to modify the response.
///
/// Authorization does not impact the query planning step. Similar to directives like `@include`
/// and `@skip`, the gateway will modify the query plan rather than the original query. So the
/// performance impact is fairly minimal.
///
/// # Example
///
/// You can initialize a new authorization extension with the Grafbase CLI:
///
/// ```bash
/// grafbase extension init --type authorization my-auth
/// ```
///
/// This will generate the following:
///
/// ```rust
/// use grafbase_sdk::{
///     AuthorizationExtension, IntoQueryAuthorization,
///     types::{SubgraphHeaders, Configuration, ErrorResponse, Token, Error, QueryElements, AuthorizationDecisions}
/// };
///
/// #[derive(AuthorizationExtension)]
/// struct MyAuth {
///     config: Config
/// }
///
/// #[derive(serde::Deserialize)]
/// struct Config {
///     my_custom_key: String
/// }
///
/// impl AuthorizationExtension for MyAuth {
///     fn new(config: Configuration) -> Result<Self, Error> {
///         let config: Config = config.deserialize()?;
///         Ok(Self { config })
///     }
///
///     fn authorize_query(
///         &mut self,
///         headers: &mut SubgraphHeaders,
///         token: Token,
///         elements: QueryElements<'_>,
///     ) -> Result<impl IntoQueryAuthorization, ErrorResponse> {
///         Ok(AuthorizationDecisions::deny_all("Unauthorized"))
///     }
/// }
/// ```
///
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
/// necessary for those. It is those directives that define the elements that must be granted
/// access by this extension. The gateway will validate that directives are correctly called. The
///
/// The simplest example would be a directive without any arguments:
///
/// ```graphql
/// directive @authorize on FIELD_DEFINITION
/// ```
///
/// Arguments can be static:
///
/// ```graphql
/// directive @authorize(meta: Metadata!) on FIELD_DEFINITION
///
/// input Metadata {
///   key: String!
/// }
/// ```
///
/// Or they can be dynamically injected from query or response data by the gateway
/// using one of the scalars defined in the [Grafbase spec](https://specs.grafbase.com/grafbase):
///
/// ```graphql
/// extend schema
///  @link(
///    url: "https://specs.grafbase.com/grafbase"
///    import: ["InputValueSet"]
///  )
///
/// directive @authorize(arguments: InputValueSet) on FIELD_DEFINITION
/// ```
///
#[allow(unused_variables)]
pub trait AuthorizationExtension: Sized + 'static {
    /// Creates a new instance of the extension. The [Configuration] will contain all the
    /// configuration defined in the `grafbase.toml` by the extension user in a serialized format.
    ///
    /// # Example
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
    fn new(config: Configuration) -> Result<Self, Error>;

    /// Authorize query elements before sending any subgraph requests. It is executed after query
    /// planning and modifies the resulting plan to minimize the performance impact. Any denied
    /// elements will *not* be requested from subgraphs.
    ///
    /// Access control should be returned with [AuthorizationDecisions] which can be constructed in
    /// multiple ways:
    /// - [AuthorizationDecisions::grant_all()] will grant access to all elements.
    /// - [AuthorizationDecisions::deny_all()] will deny access to all elements.
    /// - [AuthorizationDecisions::deny_some_builder()] creates a builder to deny some of the
    ///   elements. Elements that have not been explicitly denied will be granted access.
    ///   The simplest example being the following:
    ///
    /// ```rust
    /// # use grafbase_sdk::types::{QueryElements, ErrorResponse, AuthorizationDecisions};
    /// # fn authorize_query(
    /// #    elements: QueryElements<'_>,
    /// # ) -> Result<AuthorizationDecisions, ErrorResponse> {
    /// let mut builder = AuthorizationDecisions::deny_some_builder();
    ///
    /// for element in elements {
    ///     builder.deny(element, "Unauthorized");
    /// }
    ///
    /// Ok(builder.build())
    /// # }
    /// ```
    ///
    /// Each [QueryElement](crate::types::QueryElement) will provide:
    /// - [directive_site](crate::types::QueryElement::directive_site()) providing information on where the directive is applied, field name, etc.
    /// - [directive_arguments](crate::types::QueryElement::directive_arguments()) which similarly to the configuration
    ///   can be used to deserialize the directive arguments. The underlying format is unspecified,
    ///   but it'll always be a binary format without string escaping so it's safe to use
    ///   `[serde(borrow)] &'a str`.
    ///
    /// ```rust
    /// # use grafbase_sdk::types::{QueryElements, DirectiveSite, ErrorResponse, AuthorizationDecisions};
    /// # fn authorize_query(
    /// #    elements: QueryElements<'_>,
    /// # ) -> Result<AuthorizationDecisions, ErrorResponse> {
    /// let mut builder = AuthorizationDecisions::deny_some_builder();
    ///
    /// // For a directive like `@authorize(key: String!)`
    /// #[derive(serde::Deserialize)]
    /// struct DirectiveArguments<'a> {
    ///     #[serde(borrow)]
    ///     key: &'a str,
    /// }
    ///
    /// for element in elements {
    ///     match element.directive_site() {
    ///         DirectiveSite::FieldDefinition(field) => {
    ///             field.name();
    ///         }
    ///         _ => return Err(ErrorResponse::internal_server_error()),
    ///     }
    ///     let arguments: DirectiveArguments<'_> = element.directive_arguments()?;
    /// }
    ///
    /// Ok(builder.build())
    /// # }
    /// ```
    ///
    /// # Example
    ///
    /// Supposing the following `defintions.graphql`
    ///
    /// ```graphql
    /// directive @authorize on FIELD_DEFINITION
    /// ```
    ///
    /// With the following subgraph schema:
    ///
    /// ```graphql
    /// type Query {
    ///   user(id: ID!): User @authorize
    /// }
    ///
    /// type User {
    ///   id: ID!
    ///   name: String
    /// }
    /// ```
    ///
    /// If the client request:
    /// - `query { user(id: 1) { name } }`: the extension will be called
    ///   with a single [QueryElement](crate::types::QueryElement) with a
    ///   [FieldDefinitionDirectiveSite](crate::types::FieldDefinitionDirectiveSite).
    /// - `query { a: user(id: 1) b: user(id: 2) }`: the extension will only receive one
    ///   element if no directive argument depend on the field arguments. But if they do, through
    ///   `InputValueSet` for example, then there will be a
    ///   [QueryElement](crate::types::QueryElement) for both `a` and `b`.
    /// - `query { __typename }`: the extension is not called at all.
    ///
    /// Only elements explicitly mentioned in the query will be taken into account:
    ///
    /// ```graphql
    /// type Query {
    ///     node: Node
    /// }
    ///
    /// interface Node {
    ///    id: ID!
    /// }
    ///
    /// type User @authorize implements Node {
    ///     id: ID!
    /// }
    /// ```
    ///
    /// With a query like `query { node { id } }`, authorization will never be called even if the
    /// underlying object is a `User`.
    ///
    fn authorize_query(
        &mut self,
        ctx: &AuthenticatedRequestContext,
        headers: &mut SubgraphHeaders,
        elements: QueryElements<'_>,
    ) -> Result<impl IntoAuthorizeQueryOutput, ErrorResponse>;

    /// Authorize response elements after receiving data from subgraphs. As of today this function
    /// will be called as soon as the data is available, so if multiple response elements need
    /// authorization this method be called multiple times.
    ///
    /// This method is meant to be used with [authorize_query()](AuthorizationExtension::authorize_query()).
    /// Any element that reaches this stage will first pass through query authorization. So it must
    /// be first granted in the query stage. In addition, directive arguments are split between one
    /// that depend on response data and those that do not. [authorize_query()](AuthorizationExtension::authorize_query())
    /// will receive the latter and this function the latter.
    ///
    /// So for example with a directive defined as follows:
    /// ```graphql
    /// extend schema
    ///  @link(
    ///    url: "https://specs.grafbase.com/grafbase"
    ///    import: ["FieldSet"]
    ///  )
    ///
    /// directive @authorized(
    ///     static: String,
    ///     fields: FieldSet
    /// ) on FIELD_DEFINITION
    /// ```
    /// Used in a subgraph schema like:
    /// ```graphql
    /// type Query {
    ///     accounts: [Account!]
    /// }
    ///
    /// type Account {
    ///     id: ID!
    ///     owner: User! @authorized(static: "data", fields: "id")
    /// }
    ///
    /// type User {
    ///     id: ID!
    /// }
    /// ```
    /// Then the [authorize_query()](AuthorizationExtension::authorize_query()) method would
    /// receive `{"static": "data"}` arguments and this method would receive `{"fields": {"id":  1}}`.
    ///
    /// That's why this method receives a `state` argument provided by [authorize_query()](AuthorizationExtension::authorize_query()).
    fn authorize_response(
        &mut self,
        ctx: &AuthorizedOperationContext,
        state: Vec<u8>,
        elements: ResponseElements<'_>,
    ) -> Result<AuthorizationDecisions, Error> {
        Err("Response authorization not implemented".into())
    }
}

pub trait IntoAuthorizeQueryOutput {
    fn into_authorize_query_output(self) -> AuthorizeQueryOutput;
}

impl IntoAuthorizeQueryOutput for AuthorizeQueryOutput {
    fn into_authorize_query_output(self) -> AuthorizeQueryOutput {
        self
    }
}

impl IntoAuthorizeQueryOutput for AuthorizationDecisions {
    fn into_authorize_query_output(self) -> AuthorizeQueryOutput {
        AuthorizeQueryOutput::new(self)
    }
}

impl IntoAuthorizeQueryOutput for (Headers, AuthorizationDecisions) {
    fn into_authorize_query_output(self) -> AuthorizeQueryOutput {
        let (headers, decisions) = self;
        AuthorizeQueryOutput::new(decisions).additional_headers(headers)
    }
}

impl IntoAuthorizeQueryOutput for (AuthorizationDecisions, Headers) {
    fn into_authorize_query_output(self) -> AuthorizeQueryOutput {
        let (decisions, headers) = self;
        AuthorizeQueryOutput::new(decisions).additional_headers(headers)
    }
}

#[doc(hidden)]
pub fn register<T: AuthorizationExtension>() {
    pub(super) struct Proxy<T: AuthorizationExtension>(T);

    impl<T: AuthorizationExtension> AnyExtension for Proxy<T> {
        fn authorize_query(
            &mut self,
            ctx: &AuthenticatedRequestContext,
            headers: &mut SubgraphHeaders,
            elements: QueryElements<'_>,
        ) -> Result<AuthorizeQueryOutput, ErrorResponse> {
            self.0
                .authorize_query(ctx, headers, elements)
                .map(|output| output.into_authorize_query_output())
        }

        fn authorize_response(
            &mut self,
            ctx: &AuthorizedOperationContext,
            state: Vec<u8>,
            elements: ResponseElements<'_>,
        ) -> Result<AuthorizationDecisions, Error> {
            self.0.authorize_response(ctx, state, elements)
        }
    }

    crate::component::register_extension(Box::new(|_, config| {
        <T as AuthorizationExtension>::new(config).map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
