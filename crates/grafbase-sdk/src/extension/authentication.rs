use crate::{
    component::AnyExtension,
    types::{Configuration, Error, ErrorResponse, GatewayHeaders, PublicMetadataEndpoint, RequestContext, Token},
};

/// An authentication extension is called before any request processing, authenticating a user with
/// a token or returning an error response.
///
/// # Example
///
/// You can initialize a new authentication extension with the Grafbase CLI:
///
/// ```bash
/// grafbase extension init --type authentication my-auth
/// ```
///
/// This will generate the following:
///
/// ```rust
/// use grafbase_sdk::{
///     AuthenticationExtension,
///     types::{GatewayHeaders, Configuration, ErrorResponse, Token, Error}
/// };
///
/// #[derive(AuthenticationExtension)]
/// struct MyAuth {
///     config: Config
/// }
///
/// #[derive(serde::Deserialize)]
/// struct Config {
///     my_custom_key: String
/// }
///
/// impl AuthenticationExtension for MyAuth {
///     fn new(config: Configuration) -> Result<Self, Error> {
///         let config: Config = config.deserialize()?;
///         Ok(Self { config })
///     }
///
///     fn authenticate(&mut self, headers: &GatewayHeaders) -> Result<Token, ErrorResponse> {
///         todo!()
///     }
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
pub trait AuthenticationExtension: Sized + 'static {
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

    /// Authenticate the user with a [Token] or return an [ErrorResponse]. It is called before any
    /// GraphQL processing and an error will stop any further actions.
    ///
    /// The [HttpHeaders] are the headers received by the gateway before any header rules.
    fn authenticate(&mut self, ctx: &RequestContext, headers: &GatewayHeaders) -> Result<Token, ErrorResponse>;

    /// Define endpoints on the gateway that expose authentication related metadata. This can be used to implement [OAuth 2.0 Protected Resource Metadata](https://datatracker.ietf.org/doc/html/rfc9728), for example. This method is only called once, on gateway initialization. The endpoints are available on the gateway for GET requests at a custom path, and they return a static payload with custom headers.
    ///
    /// See the docs on [`public-metadata-endpoint`](public-metadata-endpoint) for details.
    fn public_metadata(&mut self) -> Result<Vec<PublicMetadataEndpoint>, Error> {
        Ok(vec![])
    }
}

#[doc(hidden)]
pub fn register<T: AuthenticationExtension>() {
    pub(super) struct Proxy<T: AuthenticationExtension>(T);

    impl<T: AuthenticationExtension> AnyExtension for Proxy<T> {
        fn authenticate(&mut self, headers: &GatewayHeaders) -> Result<Token, ErrorResponse> {
            self.0.authenticate(&RequestContext, headers)
        }

        fn public_metadata(&mut self) -> Result<Vec<PublicMetadataEndpoint>, Error> {
            AuthenticationExtension::public_metadata(&mut self.0)
        }
    }

    crate::component::register_extension(Box::new(|_, config| {
        <T as AuthenticationExtension>::new(config).map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
