use crate::{types::Headers, wit};

use super::Error;

/// Error identifier to allow re-using the same error for multiple elements. In the gateway
/// response, the error will be repeated if necessary during serialization.
#[derive(Clone, Copy)]
pub struct ErrorId(u32);

/// Output type for the [authorize_query()](crate::AuthorizationExtension::authorize_query())
/// method.
pub struct AuthorizeQueryOutput {
    /// Authorization decisions for each query element to be applied by the GraphQL engine.
    pub(crate) decisions: AuthorizationDecisions,
    /// Authorization context if any.
    pub(crate) context: Vec<u8>,
    /// Authorization state if any.
    pub(crate) state: Vec<u8>,
    /// Additional headers to add to the subgraph headers if any.
    pub(crate) additional_headers: Option<Headers>,
}

impl AuthorizeQueryOutput {
    /// Create a new `AuthorizeQueryOutput` with the given decisions.
    pub fn new(decisions: AuthorizationDecisions) -> Self {
        Self {
            decisions,
            context: Vec::new(),
            state: Vec::new(),
            additional_headers: None,
        }
    }

    /// Set the authorization context for the request and extension.
    /// Accessible by other extensions.
    pub fn context(mut self, context: impl Into<Vec<u8>>) -> Self {
        self.context = context.into();
        self
    }

    /// Set the authorization state for the request.
    /// Only accessible by [authorize_response()](crate::AuthorizationExtension::authorize_response())
    /// of the same extensions.
    pub fn state(mut self, state: impl Into<Vec<u8>>) -> Self {
        self.state = state.into();
        self
    }

    /// Set additional headers to be added to the subgraph headers.
    pub fn additional_headers(mut self, headers: Headers) -> Self {
        self.additional_headers = Some(headers);
        self
    }
}

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

    /// Create a `DenySomeBuilder` best suited to deny some elements. By
    /// default, all elements are granted access.
    pub fn deny_some_builder() -> DenySomeBuilder {
        DenySomeBuilder(wit::AuthorizationDecisionsDenySome {
            element_to_error: Vec::new(),
            errors: Vec::new(),
        })
    }
}

/// To be used when denying some of the elements. By default everything is granted.
pub struct DenySomeBuilder(wit::AuthorizationDecisionsDenySome);

impl DenySomeBuilder {
    /// Deny access to the specified element in the query with the specified error.
    pub fn deny(&mut self, x: impl private::QueryElementOrResponseItem, error: impl Into<Error>) {
        let error_id = self.push_error(error);
        self.deny_with_error_id(x, error_id)
    }

    /// Deny access to the specified element in the query, re-using an existing error.
    pub fn deny_with_error_id(&mut self, x: impl private::QueryElementOrResponseItem, error_id: ErrorId) {
        self.0.element_to_error.push((x.ix(), error_id.0));
    }

    /// Returns an ErrorId that can be used to reference this error later in `deny_with_error_id`.
    pub fn push_error(&mut self, error: impl Into<Error>) -> ErrorId {
        let error_ix = self.0.errors.len() as u32;
        self.0.errors.push(Into::<Error>::into(error).into());
        ErrorId(error_ix)
    }

    /// Build the final AuthorizationDecisions
    pub fn build(self) -> AuthorizationDecisions {
        AuthorizationDecisions(wit::AuthorizationDecisions::DenySome(self.0))
    }
}

pub(super) mod private {
    /// Either a `QueryElement` or a `ResponseItem`.
    pub trait QueryElementOrResponseItem: crate::sealed::Sealed {
        #[doc(hidden)]
        fn ix(&self) -> u32;
    }
}
