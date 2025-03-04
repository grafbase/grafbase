use crate::wit;

use super::{Error, QueryElement};

/// Error identifier to allow re-using the same error for multiple elements. In the gateway
/// response, the error will be repeated if necessary during serialization.
#[derive(Clone, Copy)]
pub struct ErrorId(u32);

/// Authorization decisions for each query elements to be applied by the GraphQL engine.
pub struct AuthorizationDecisions(wit::AuthorizationDecisions);

impl From<AuthorizationDecisions> for wit::AuthorizationDecisions {
    fn from(value: AuthorizationDecisions) -> Self {
        value.0
    }
}

impl AuthorizationDecisions {
    /// Grant access all elements in the query.
    pub fn grant_all() -> Self {
        Self(wit::AuthorizationDecisions::GrantAll)
    }

    /// Deny access to all elements in the query with the specified error
    pub fn deny_all(error: impl Into<Error>) -> Self {
        Self(wit::AuthorizationDecisions::DenyAll(Into::<Error>::into(error).into()))
    }

    /// Create a `SparseDenyAuthorizationDecisionsBuilder` best suited to deny some elements. By
    /// default, all elements are granted access.
    pub fn sparse_deny() -> SparseDenyAuthorizationDecisions {
        SparseDenyAuthorizationDecisions(wit::SparseDenyAuthorizationDecisions {
            element_to_error: Vec::new(),
            errors: Vec::new(),
        })
    }
}

/// To be used when denying some of the elements. By default everything is granted.
pub struct SparseDenyAuthorizationDecisions(wit::SparseDenyAuthorizationDecisions);

impl SparseDenyAuthorizationDecisions {
    /// Deny access to the specified element in the query with the specified error.
    pub fn deny(&mut self, element: QueryElement<'_>, error: impl Into<Error>) {
        let error_id = self.push_error(error);
        self.deny_with_error_id(element, error_id)
    }

    /// Deny access to the specified element in the query, re-using an existing error.
    pub fn deny_with_error_id(&mut self, element: QueryElement<'_>, error_id: ErrorId) {
        self.0.element_to_error.push((element.ix, error_id.0));
    }

    /// Returns an ErrorId that can be used to reference this error later in `deny_with_error_id`.
    pub fn push_error(&mut self, error: impl Into<Error>) -> ErrorId {
        let error_ix = self.0.errors.len() as u32;
        self.0.errors.push(Into::<Error>::into(error).into());
        ErrorId(error_ix)
    }
}

impl From<SparseDenyAuthorizationDecisions> for AuthorizationDecisions {
    fn from(value: SparseDenyAuthorizationDecisions) -> Self {
        AuthorizationDecisions(wit::AuthorizationDecisions::SparseDeny(value.0))
    }
}
