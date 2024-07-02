use std::fmt;

use registry_v2::IdRange;

use crate::{
    ids::{StringId, SupertypeId},
    IdReader, Iter, ReadContext, RecordLookup, RegistryId,
};

impl super::PartialCacheRegistry {
    pub fn supertypes<'a>(&'a self, typename: &str) -> Iter<'a, Supertype<'a>> {
        Iter::new(self.supertype_range(typename), self)
    }

    pub fn is_supertype(&self, possible_supertype: &str, subtype: &str) -> bool {
        let Some(possible_supertype_id) = self.strings.get_index_of(possible_supertype).map(StringId::new) else {
            return false;
        };
        let supertype_range = self.supertype_range(subtype);

        self.supertypes[supertype_range.start.to_index()..supertype_range.end.to_index()]
            .binary_search(&possible_supertype_id)
            .is_ok()
    }

    fn supertype_range(&self, typename: &str) -> IdRange<SupertypeId> {
        self.strings
            .get_index_of(typename)
            .map(StringId::new)
            .and_then(|string_id| self.type_relations.get(&string_id))
            .copied()
            .unwrap_or_default()
    }
}

pub struct Supertype<'a>(pub(crate) ReadContext<'a, SupertypeId>);

impl<'a> Supertype<'a> {
    pub fn typename(&self) -> &'a str {
        let registry = self.0.registry;
        registry.lookup(*registry.lookup(self.0.id))
    }
}

impl fmt::Debug for Supertype<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Supertype").field("typename", &self.typename()).finish()
    }
}

impl RegistryId for SupertypeId {
    type Reader<'a> = Supertype<'a>;
}

impl IdReader for Supertype<'_> {
    type Id = SupertypeId;
}

impl<'a> From<ReadContext<'a, SupertypeId>> for Supertype<'a> {
    fn from(value: ReadContext<'a, SupertypeId>) -> Self {
        Self(value)
    }
}
