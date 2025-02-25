use crate::{
    component::AnyExtension,
    types::{ErrorResponse, QueryAuthorization, QueryElements, ResponseAuthorization, ResponseElements},
    wit, SharedContext,
};

use super::Extension;

/// A trait that extends `Extension` and provides authorization functionality.
pub trait Authorizer: Extension {
    /// Authorize query elements before sending any subgraph requests.
    /// The query elements will contain every element in the operation with a definition annotated
    /// with one of the extension's authorization directive. This naturally includes fields, but
    /// also objects, interfaces, unions, enums and scalars.
    ///
    /// Only elements explicitly mentioned in the query will be taken into account. Authorization
    /// on a object behind an interface won't be called if it's not explicitly mentioned, so if
    /// only interface fields are used.
    fn authorize_query<'a>(
        &'a mut self,
        context: SharedContext,
        elements: QueryElements<'a>,
    ) -> Result<QueryAuthorization<Box<dyn ResponseAuthorizer<'a>>>, ErrorResponse>;
}

pub trait ResponseAuthorizer<'a>: 'a {
    /// Authorize response elements after receiving a subgraph response.
    ///
    /// Only elements explicitly mentioned in the query will be taken into account. Authorization
    /// on a object behind an interface won't be called if it's not explicitly mentioned, so if
    /// only interface fields are used.
    fn authorize_response(&mut self, elements: ResponseElements<'_>) -> Result<ResponseAuthorization, wit::Error>;
}

impl<'a, T: ResponseAuthorizer<'a>> ResponseAuthorizer<'a> for Box<T> {
    fn authorize_response(&mut self, elements: ResponseElements<'_>) -> Result<ResponseAuthorization, wit::Error> {
        self.as_mut().authorize_response(elements)
    }
}

#[doc(hidden)]
pub fn register<T: Authorizer>() {
    pub(super) struct Proxy<T: Authorizer>(T);

    impl<T: Authorizer> AnyExtension for Proxy<T> {
        fn authorize_query<'a>(
            &'a mut self,
            context: SharedContext,
            elements: QueryElements<'a>,
        ) -> Result<QueryAuthorization<Box<dyn ResponseAuthorizer<'a>>>, ErrorResponse> {
            Authorizer::authorize_query(&mut self.0, context, elements)
        }
    }

    crate::component::register_extension(Box::new(|schema_directives, config| {
        <T as Extension>::new(schema_directives, config)
            .map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
