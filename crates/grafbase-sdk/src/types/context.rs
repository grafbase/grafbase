use crate::{SdkError, types::Token};

/// Context available after the [on_request()](crate::HooksExtension::on_request()) hook.
pub struct RequestContext;

impl RequestContext {
    /// Returns the Hook state created by the [on_request()](crate::HooksExtension::on_request())
    /// hook if any.
    pub fn hook_state(&self) -> Vec<u8> {
        crate::component::current_context().hook_state()
    }
}

/// Context available while processing a GraphQL operation.
pub struct OperationContext;

impl OperationContext {
    /// Hook state created by the [on_request()](crate::HooksExtension::on_request()) hook if any.
    pub fn hook_state(&self) -> Vec<u8> {
        crate::component::current_context().hook_state()
    }

    /// Authentication token provided by an authentication extension if any.
    pub fn authentication_token(&self) -> Token {
        crate::component::current_context().authentication_token().into()
    }

    /// Retrieve the current authorization state if any.
    /// This method will fail if there is more one authorization state, from different extensions.
    pub fn authorization_state(&self) -> Result<Vec<u8>, SdkError> {
        crate::component::current_context()
            .authorization_state(None)
            .map_err(Into::into)
    }

    /// Retrieve the current authorization state for a given extension.
    /// The key must match the one used in the configuration.
    /// Fails if the key doesn't point to an authorization extension.
    ///
    /// Use [authorization_state()](OperationContext::authorization_state()) if you have only one
    /// authorization extension returning a non-empty state.
    pub fn authorization_state_by_key(&self, key: &str) -> Result<Vec<u8>, SdkError> {
        crate::component::current_context()
            .authorization_state(Some(key))
            .map_err(Into::into)
    }
}
