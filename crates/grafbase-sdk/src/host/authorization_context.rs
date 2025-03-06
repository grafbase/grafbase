use crate::{wit, Token};

/// Context available for authorization
pub struct AuthorizationContext(wit::AuthorizationContext);

impl From<wit::AuthorizationContext> for AuthorizationContext {
    fn from(context: wit::AuthorizationContext) -> Self {
        Self(context)
    }
}

impl AuthorizationContext {
    /// Gateway HTTP headers.
    pub fn headers(&self) -> super::Headers {
        self.0.headers().into()
    }

    /// Token produced by an authentication extension.
    pub fn token(&self) -> Token {
        self.0.token().into()
    }
}
