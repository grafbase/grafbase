use grafbase_sdk::{
    AuthenticationExtension, Error, GatewayHeaders, Token,
    types::{Configuration, ErrorResponse, StatusCode},
};

/// Dummy extension serving as a placeholder for test extension written directly inside the
/// integration-tests. It allows us to have a single ExtensionCatalog for both and share the
/// ExtensionId space.
#[derive(AuthenticationExtension)]
struct Placeholder;

impl AuthenticationExtension for Placeholder {
    fn new(_: Configuration) -> Result<Self, Error> {
        Ok(Self)
    }

    fn authenticate(&mut self, _headers: &GatewayHeaders) -> Result<Token, ErrorResponse> {
        Err(ErrorResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            .with_error("This extension should never be called. It means the extension dispatch didn't work."))
    }
}
