use crate::{
    component::AnyExtension,
    types::{ErrorResponse, Token},
    wit::Headers,
    SharedContext,
};

use super::Extension;

/// A trait that extends `Extension` and provides authentication functionality.
pub trait Authorizer: Extension {
    /// Authorize query elements before query execution and return a response
    /// authorized if relevant.
    fn authorize_query(
        &mut self,
        context: SharedContext,
        resources: Vec<QueryResource>,
    ) -> Result<QueryAuthorization<impl ResponseAuthorizer<'_>>, ErrorResponse>;
}

struct QueryResource {
  directive_name: String,
  sites: Vec<TypeSystemDirectiveSite>
}

struct ResponseResource {
  directive_name: String,
  elements: Vec<(TypeSystemDirectiveSite, Vec<Vec<u8>>)>
}

enum TypeSystemDirectiveSite {
  Object(ObjectSite)
  FieldDefinition(FieldDefinitionSite)
  Interface(InterfaceSite)
  Union(UnionSite)
}

struct FieldDefinitionDirectiveSite {
  parent_type_name: String,
  field_name: String,
  arguments: Vec<u8>
}

struct ObjectDirectiveSite {
  object_name: String,
  arguments: Vec<u8>
}

struct UnionDirectiveSite {
  union_name: String,
  arguments: Vec<u8>
}

struct InterfaceDirectiveSite {
  interface_name: String,
  arguments: Vec<u8>
}


#[doc(hidden)]
pub fn register<T: Authorizer>() {
    pub(super) struct Proxy<T: Authorizer>(T);

    impl<T: Authorizer> AnyExtension for Proxy<T> {}

    crate::component::register_extension(Box::new(|schema_directives, config| {
        <T as Extension>::new(schema_directives, config)
            .map(|extension| Box::new(Proxy(extension)) as Box<dyn AnyExtension>)
    }))
}
