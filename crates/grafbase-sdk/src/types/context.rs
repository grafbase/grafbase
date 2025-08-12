use crate::{SdkError, types::Token, wit};

/// Context available after the [on_request()](crate::HooksExtension::on_request()) hook.
pub struct RequestContext(wit::RequestContext);

impl From<wit::RequestContext> for RequestContext {
    fn from(context: wit::RequestContext) -> Self {
        Self(context)
    }
}

impl RequestContext {
    /// Returns the Hook context created by the [on_request()](crate::HooksExtension::on_request())
    /// hook if any.
    pub fn hook_context(&self) -> Vec<u8> {
        self.0.hook_context()
    }
}

/// Context available after the [authenticate()](crate::AuthenticationExtension::authenticate())
pub struct AuthenticatedRequestContext(wit::AuthenticatedRequestContext);

impl From<wit::AuthenticatedRequestContext> for AuthenticatedRequestContext {
    fn from(context: wit::AuthenticatedRequestContext) -> Self {
        Self(context)
    }
}

impl AuthenticatedRequestContext {
    /// Returns the Hook context created by the [on_request()](crate::HooksExtension::on_request())
    /// hook if any.
    pub fn hook_context(&self) -> Vec<u8> {
        self.0.hook_context()
    }
    /// Returns the authentication token provided by an authentication extension if any.
    pub fn token(&self) -> Token {
        self.0.token().into()
    }
}

/// Context available after the [authorize_query()](crate::AuthorizationExtension::authorize_query())
pub struct AuthorizedOperationContext(wit::AuthorizedOperationContext);

impl From<wit::AuthorizedOperationContext> for AuthorizedOperationContext {
    fn from(context: wit::AuthorizedOperationContext) -> Self {
        Self(context)
    }
}

impl AuthorizedOperationContext {
    /// Returns the Hook context created by the [on_request()](crate::HooksExtension::on_request())
    /// hook if any.
    pub fn hook_context(&self) -> Vec<u8> {
        self.0.hook_context()
    }
    /// Returns the authentication token provided by an authentication extension if any.
    pub fn token(&self) -> Token {
        self.0.token().into()
    }

    /// Retrieve the current authorization context if any.
    /// This method will fail if there is more one authorization context, from different extensions.
    pub fn authorization_context(&self) -> Result<Vec<u8>, SdkError> {
        self.0.authorization_context(None).map_err(Into::into)
    }

    /// Retrieve the current authorization state for a given extension.
    /// The key must match the one used in the configuration.
    /// Fails if the key doesn't point to an authorization extension.
    ///
    /// Use [authorization_context()](AuthorizedOperationContext::authorization_context()) if you have only one
    /// authorization extension returning a non-empty state.
    pub fn authorization_icontext_by_key(&self, key: &str) -> Result<Vec<u8>, SdkError> {
        self.0.authorization_context(Some(key)).map_err(Into::into)
    }
}
