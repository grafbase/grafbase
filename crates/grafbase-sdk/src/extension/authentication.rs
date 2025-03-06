use crate::{
    component::AnyExtension,
    host::Headers,
    types::{Configuration, ErrorResponse, Token},
    Error,
};

/// A trait that extends `Extension` and provides authentication functionality.
pub trait AuthenticationExtension: Sized + 'static {
    /// Creates a new instance of the extension.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for this extension, from the gateway TOML.
    ///
    /// # Returns
    ///
    /// Returns an instance of this resolver. Upon failure, every call to this extension will fail.
    fn new(config: Configuration) -> Result<Self, Error>;

    /// Authenticates the request using the provided headers.
    ///
    /// # Arguments
    /// * `headers` - The request headers to authenticate with.
    ///
    /// # Returns
    /// * `Ok(Token)` - A valid authentication token if successful.
    /// * `Err(ErrorResponse)` - An error response if authentication fails.
    fn authenticate(&mut self, headers: Headers) -> Result<Token, ErrorResponse>;
}

#[doc(hidden)]
pub fn register<T: AuthenticationExtension>() {
    pub(super) struct Proxy<T: AuthenticationExtension>(T);

    impl<T: AuthenticationExtension> AnyExtension for Proxy<T> {
        fn authenticate(&mut self, headers: Headers) -> Result<Token, ErrorResponse> {
            AuthenticationExtension::authenticate(&mut self.0, headers)
        }
    }

    crate::component::register_extension(Box::new(|_, config| {
        <T as AuthenticationExtension>::new(config).map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
