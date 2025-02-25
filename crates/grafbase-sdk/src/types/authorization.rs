use crate::{extension::authorization::ResponseAuthorizer, wit};

use super::{QueryElement, QueryElements, ResponseElement, ResponseElements};

/// Authorization decisions for each query elements to be applied by the GraphQL engine.
pub struct QueryAuthorization<A> {
    pub(crate) decisions: wit::AuthorizationDecisions,
    pub(crate) response_authorizer: Option<A>,
}

/// Error identifier to allow re-using the same error for multiple elements. In the gateway
/// response, the error will be repeated if necessary during serialization.
#[derive(Clone, Copy)]
pub struct ErrorId(u32);

impl<A> QueryAuthorization<A> {
    /// Initialize the query authorization from the query elements.
    pub fn new_for(elements: QueryElements<'_>) -> Self {
        Self {
            decisions: wit::AuthorizationDecisions {
                granted_bitset: vec![0; 1 + elements.len() / 8],
                element_ix_to_error_ix: Vec::new(),
                errors: Vec::new(),
            },
            response_authorizer: None,
        }
    }

    /// Grant access all elements in the query.
    pub fn grant_all(&mut self) {
        self.decisions.granted_bitset.iter_mut().for_each(|byte| *byte = 0xff);
    }

    /// Deny access to all elements in the query with the specified error.
    pub fn deny_all(&mut self, err: impl Into<wit::Error>) {
        self.deny_all_without_error();
        let error_id = self.push_error(err.into());
        self.decisions.element_ix_to_error_ix = (0..self.decisions.element_ix_to_error_ix.len())
            .map(|ix| (ix as u32, error_id.0))
            .collect();
    }

    /// Deny access to all elements in the query without an error.
    /// Elements within lists will simply be removed instead of propagating an error.
    pub fn deny_all_without_error(&mut self) {
        self.decisions.granted_bitset.iter_mut().for_each(|byte| *byte = 0x00);
    }

    /// Grant access to the specified element in the query.
    pub fn grant(&mut self, element: QueryElement<'_>) {
        let ix = element.ix as usize;
        self.decisions.granted_bitset[ix / 8] |= 1 << (ix % 8);
    }

    /// Deny access to the specified element in the query with the specified error.
    pub fn deny(&mut self, element: QueryElement<'_>, error: impl Into<wit::Error>) {
        let ix = self.decisions.errors.len() as u32;
        self.decisions.errors.push(error.into());
        self.deny_with_error_id(element, ErrorId(ix))
    }

    /// Deny access to the specified element in the query, re-using an existing error.
    pub fn deny_with_error_id(&mut self, element: QueryElement<'_>, error_id: ErrorId) {
        self.deny_without_error(element);
        self.decisions.element_ix_to_error_ix.push((element.ix, error_id.0));
    }

    /// Deny access to the specified element in the query without an error.
    /// If this element is within a list, it'll simply be removed without propagating any error.
    pub fn deny_without_error(&mut self, element: QueryElement<'_>) {
        let ix = element.ix as usize;
        self.decisions.granted_bitset[ix / 8] &= !(1 << (ix % 8));
    }

    /// Returns an ErrorId that can be used to reference this error later in `deny_with_error_id`.
    pub fn push_error(&mut self, error: impl Into<wit::Error>) -> ErrorId {
        let error_ix = self.decisions.errors.len() as u32;
        self.decisions.errors.push(error.into());
        ErrorId(error_ix)
    }

    /// A `ResponseAuthorizer` allows taking a decision based on response data when it appears.
    /// If provided it'll be called if necessary for each subgraph response.
    pub fn with_response_authorizer<'a>(
        self,
        response_authorizer: impl ResponseAuthorizer<'a>,
    ) -> QueryAuthorization<Box<dyn ResponseAuthorizer<'a>>> {
        QueryAuthorization {
            decisions: self.decisions,
            response_authorizer: Some(Box::new(response_authorizer)),
        }
    }
}

/// Authorization decisions for each response elements, when it depends on response data, to be applied by the GraphQL engine.
pub struct ResponseAuthorization {
    pub(crate) decisions: wit::AuthorizationDecisions,
}

impl ResponseAuthorization {
    /// Initialize the response authorization from the response elements
    pub fn new_for(elements: ResponseElements<'_>) -> Self {
        Self {
            decisions: wit::AuthorizationDecisions {
                granted_bitset: vec![0; 1 + elements.len() / 8],
                element_ix_to_error_ix: Vec::new(),
                errors: Vec::new(),
            },
        }
    }

    /// Grant access all elements in the response.
    pub fn grant_all(&mut self) {
        self.decisions.granted_bitset.iter_mut().for_each(|byte| *byte = 0xff);
    }

    /// Deny access to all elements in the response with the specified error.
    pub fn deny_all(&mut self, err: impl Into<wit::Error>) {
        self.deny_all_without_error();
        let error_id = self.push_error(err.into());
        self.decisions.element_ix_to_error_ix = (0..self.decisions.element_ix_to_error_ix.len())
            .map(|ix| (ix as u32, error_id.0))
            .collect();
    }

    /// Deny access to all elements in the response without an error.
    /// Elements within lists will simply be removed instead of propagating an error.
    pub fn deny_all_without_error(&mut self) {
        self.decisions.granted_bitset.iter_mut().for_each(|byte| *byte = 0x00);
    }

    /// Grant access to the specified response element.
    pub fn grant(&mut self, element: ResponseElement<'_>) {
        let ix = element.ix as usize;
        self.decisions.granted_bitset[ix / 8] |= 1 << (ix % 8);
    }

    /// Deny access to the specified response element with the provided error.
    pub fn deny(&mut self, element: ResponseElement<'_>, error: impl Into<wit::Error>) {
        let ix = self.decisions.errors.len() as u32;
        self.decisions.errors.push(error.into());
        self.deny_with_error_id(element, ErrorId(ix))
    }

    /// Deny access to the specified response element, reusing an existing error ID.
    pub fn deny_with_error_id(&mut self, element: ResponseElement<'_>, error_id: ErrorId) {
        self.deny_without_error(element);
        self.decisions.element_ix_to_error_ix.push((element.ix, error_id.0));
    }

    /// Deny access to the specified response element without providing an error.
    /// If this element is within a list, it'll simply be removed without propagating any error.
    pub fn deny_without_error(&mut self, element: ResponseElement<'_>) {
        let ix = element.ix as usize;
        self.decisions.granted_bitset[ix / 8] &= !(1 << (ix % 8));
    }

    /// Returns an ErrorId that can be used to reference this error later in `deny_with_error_id`.
    pub fn push_error(&mut self, error: impl Into<wit::Error>) -> ErrorId {
        let error_ix = self.decisions.errors.len() as u32;
        self.decisions.errors.push(error.into());
        ErrorId(error_ix)
    }
}
