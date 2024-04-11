use id_newtypes::IdRange;

use crate::{CacheControl, Deprecated, RequiredScopesWalker, SchemaWalker, TypeSystemDirective, TypeSystemDirectiveId};

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
}
