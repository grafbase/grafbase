use std::{iter, sync::Arc};

use registry_for_cache::PartialCacheRegistry;

/// This trait abstracts over the Registry to provide subtype to partial-caching.
///
/// This crate does currently pull in the partial-caching registry, so it could
/// directly use the registry.  But I think there's some value in a trait anyway:
/// lets tests skip the registry and potentially makes it easier to use this crate
/// elsewhere in the future.
pub trait TypeRelationships: Send + Sync {
    fn type_condition_matches(&self, type_condition: &str, typename: &str) -> bool;

    fn supertypes<'b>(&'b self, typename: &str) -> Box<dyn Iterator<Item = &str> + 'b>;
}

// TODO: Possibly rename this...

impl TypeRelationships for PartialCacheRegistry {
    fn type_condition_matches(&self, type_condition: &str, typename: &str) -> bool {
        if type_condition == typename {
            return true;
        }
        // TODO: Are these arguments the right way round...?
        self.is_supertype(type_condition, typename)
    }

    fn supertypes<'b>(&'b self, typename: &str) -> Box<dyn Iterator<Item = &'b str> + 'b> {
        Box::new(PartialCacheRegistry::supertypes(self, typename).map(|supertype| supertype.typename()))
    }
}

pub struct NoSubtypes;

/// Used by tests to simulate a SubtypeInfo with no Subtypes
pub fn no_subtypes() -> Arc<dyn TypeRelationships> {
    Arc::new(NoSubtypes)
}

impl TypeRelationships for NoSubtypes {
    fn type_condition_matches(&self, type_condition: &str, typename: &str) -> bool {
        type_condition == typename
    }

    fn supertypes<'b>(&'b self, _typename: &str) -> Box<dyn Iterator<Item = &str> + 'b> {
        Box::new(iter::empty())
    }
}
