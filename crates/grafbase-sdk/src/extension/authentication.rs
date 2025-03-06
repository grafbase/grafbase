use crate::{
    component::AnyExtension,
    host::Headers,
    types::{ErrorResponse, Token},
};

use super::Extension;

/// A trait that extends `Extension` and provides authentication functionality.
pub trait AuthenticationExtension: Extension {
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

    crate::component::register_extension(Box::new(|schema_directives, config| {
        <T as Extension>::new(schema_directives, config)
            .map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
