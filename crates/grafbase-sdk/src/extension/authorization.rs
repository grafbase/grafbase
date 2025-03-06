use crate::{
    component::AnyExtension,
    host::AuthorizationContext,
    types::{AuthorizationDecisions, Configuration, ErrorResponse, QueryElements},
    Error,
};

/// A trait that extends `Extension` and provides authorization functionality.
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
        ctx: AuthorizationContext,
        elements: QueryElements<'_>,
    ) -> Result<impl Into<AuthorizationDecisions>, ErrorResponse>;
}

#[doc(hidden)]
pub fn register<T: AuthorizationExtension>() {
    pub(super) struct Proxy<T: AuthorizationExtension>(T);

    impl<T: AuthorizationExtension> AnyExtension for Proxy<T> {
        fn authorize_query(
            &mut self,
            ctx: AuthorizationContext,
            elements: QueryElements<'_>,
        ) -> Result<AuthorizationDecisions, ErrorResponse> {
            AuthorizationExtension::authorize_query(&mut self.0, ctx, elements).map(Into::into)
        }
    }

    crate::component::register_extension(Box::new(|_, config| {
        <T as AuthorizationExtension>::new(config).map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
