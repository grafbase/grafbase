use id_newtypes::IdRange;

use crate::{
    AuthorizedDirectiveId, CacheControl, Deprecated, InputValueSet, RequiredFieldSet, RequiredScopesWalker,
    SchemaInputValueWalker, SchemaWalker, TypeSystemDirective, TypeSystemDirectiveId,
};

pub type TypeSystemDirectivesWalker<'a> = SchemaWalker<'a, IdRange<TypeSystemDirectiveId>>;

impl<'a> TypeSystemDirectivesWalker<'a> {
    pub fn cache_control(&self) -> Option<&'a CacheControl> {
        self.as_ref().iter().find_map(|d| match d {
            TypeSystemDirective::CacheControl(id) => Some(&self.schema[*id]),
            _ => None,
        })
    }

    pub fn has_deprecated(&self) -> bool {
        self.as_ref()
            .iter()
            .any(|d| matches!(d, TypeSystemDirective::Deprecated(_)))
    }

    pub fn deprecated(&self) -> Option<&'a Deprecated> {
        self.as_ref().iter().find_map(|d| match d {
            TypeSystemDirective::Deprecated(deprecated) => Some(deprecated),
            _ => None,
        })
    }

    pub fn has_authenticated(&self) -> bool {
        self.as_ref()
            .iter()
            .any(|d| matches!(d, TypeSystemDirective::Authenticated))
    }

    pub fn requires_scopes(&self) -> Option<RequiredScopesWalker<'a>> {
        self.as_ref().iter().find_map(|d| match d {
            TypeSystemDirective::RequiresScopes(id) => Some(self.walk(*id)),
            _ => None,
        })
    }

    pub fn iter_required_fields(&self) -> impl Iterator<Item = &'a RequiredFieldSet> + 'a {
        let schema = self.schema;
        self.as_ref().iter().filter_map(|d| match d {
            TypeSystemDirective::Authorized(id) => {
                let directive = &schema[*id];
                directive.fields.map(|id| &schema[id])
            }
            _ => None,
        })
    }

    pub fn any_has_required_fields(&self) -> bool {
        self.as_ref().iter().any(|d| match d {
            TypeSystemDirective::Authorized(id) => {
                let directive = &self.schema[*id];
                directive.fields.is_some()
            }
            _ => false,
        })
    }

    pub fn authorized(&self) -> impl Iterator<Item = AuthorizedDirectiveWalker<'a>> + 'a {
        let schema = self.schema;
        self.as_ref().iter().filter_map(move |d| match d {
            TypeSystemDirective::Authorized(id) => Some(schema.walk(*id)),
            _ => None,
        })
    }
}

pub type AuthorizedDirectiveWalker<'a> = SchemaWalker<'a, AuthorizedDirectiveId>;

impl<'a> AuthorizedDirectiveWalker<'a> {
    pub fn arguments(&self) -> &'a InputValueSet {
        &self.as_ref().arguments
    }

    pub fn node(&self) -> &'a RequiredFieldSet {
        self.as_ref()
            .node
            .map(|id| &self.schema[id])
            .unwrap_or(&crate::requires::EMPTY)
    }

    pub fn metadata(&self) -> Option<SchemaInputValueWalker<'a>> {
        self.as_ref()
            .metadata
            .map(|id| self.walk(&self.schema.graph.input_values[id]))
    }
}
