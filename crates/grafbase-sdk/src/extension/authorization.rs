use crate::{
    component::AnyExtension,
    host::SubgraphHeaders,
    types::{AuthorizationDecisions, Configuration, ErrorResponse, QueryElements, ResponseElements},
    Error, Token,
};

/// A trait that extends `Extension` and provides authorization functionality.
#[allow(unused_variables)]
pub trait AuthorizationExtension: Sized + 'static {
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

    /// Authorize query elements before sending any subgraph requests.
    /// The query elements will contain every element in the operation with a definition annotated
    /// with one of the extension's authorization directive. This naturally includes fields, but
    /// also objects, interfaces, unions, enums and scalars.
    ///
    /// Only elements explicitly mentioned in the query will be taken into account. Authorization
    /// on a object behind an interface won't be called if it's not explicitly mentioned, so if
    /// only interface fields are used.
    fn authorize_query(
        &mut self,
        headers: &mut SubgraphHeaders,
        token: Token,
        elements: QueryElements<'_>,
    ) -> Result<impl IntoQueryAuthorization, ErrorResponse>;

    fn authorize_response(
        &mut self,
        state: Vec<u8>,
        elements: ResponseElements<'_>,
    ) -> Result<AuthorizationDecisions, Error> {
        Err("Response authorization not implemented".into())
    }
}

pub trait IntoQueryAuthorization {
    fn into_query_authorization(self) -> (AuthorizationDecisions, Vec<u8>);
}

impl IntoQueryAuthorization for AuthorizationDecisions {
    fn into_query_authorization(self) -> (AuthorizationDecisions, Vec<u8>) {
        (self, Vec::new())
    }
}

impl IntoQueryAuthorization for (AuthorizationDecisions, Vec<u8>) {
    fn into_query_authorization(self) -> (AuthorizationDecisions, Vec<u8>) {
        self
    }
}

#[doc(hidden)]
pub fn register<T: AuthorizationExtension>() {
    pub(super) struct Proxy<T: AuthorizationExtension>(T);

    impl<T: AuthorizationExtension> AnyExtension for Proxy<T> {
        fn authorize_query(
            &mut self,
            headers: &mut SubgraphHeaders,
            token: Token,
            elements: QueryElements<'_>,
        ) -> Result<(AuthorizationDecisions, Vec<u8>), ErrorResponse> {
            AuthorizationExtension::authorize_query(&mut self.0, headers, token, elements)
                .map(IntoQueryAuthorization::into_query_authorization)
        }

        fn authorize_response(
            &mut self,
            state: Vec<u8>,
            elements: ResponseElements<'_>,
        ) -> Result<AuthorizationDecisions, Error> {
            AuthorizationExtension::authorize_response(&mut self.0, state, elements)
        }
    }

    crate::component::register_extension(Box::new(|_, config| {
        <T as AuthorizationExtension>::new(config).map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
