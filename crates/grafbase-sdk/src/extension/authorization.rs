use crate::{
    component::AnyExtension,
    types::{ErrorResponse, Token},
};

use super::Extension;

/// A trait that extends `Extension` and provides authentication functionality.
pub trait Authorizer: Extension {}

#[doc(hidden)]
pub fn register<T: Authorizer>() {
    pub(super) struct Proxy<T: Authorizer>(T);

    impl<T: Authorizer> AnyExtension for Proxy<T> {}

    crate::component::register_extension(Box::new(|schema_directives, config| {
        <T as Extension>::new(schema_directives, config)
            .map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
