use super::prelude::*;
use super::{
    metatype::MetaType,
    prelude::ids::{MetaTypeId, UnionTypeId},
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct UnionTypeRecord {
    #[serde(rename = "0")]
    pub name: StringId,
    #[serde(rename = "1", skip_serializing_if = "Option::is_none", default)]
    pub description: Option<StringId>,
    #[serde(rename = "2", skip_serializing_if = "crate::Container::is_empty", default)]
    pub possible_types: Vec<MetaTypeId>,
    #[serde(rename = "3")]
    pub discriminators: UnionDiscriminators,
}

#[derive(Clone, Copy)]
pub struct UnionType<'a>(pub(crate) ReadContext<'a, UnionTypeId>);

impl<'a> UnionType<'a> {
    pub fn name(&self) -> &'a str {
        let registry = &self.0.registry;
        registry.lookup(registry.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).description.map(|id| registry.lookup(id))
    }
    pub fn possible_types(&self) -> impl ExactSizeIterator<Item = MetaType<'a>> + 'a {
        let registry = self.0.registry;
        registry
            .lookup(self.0.id)
            .possible_types
            .iter()
            .map(|id| registry.read(*id))
    }
    pub fn discriminators(&self) -> &'a UnionDiscriminators {
        let registry = self.0.registry;
        &registry.lookup(self.0.id).discriminators
    }
}

impl fmt::Debug for UnionType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnionType")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("possible_types", &self.possible_types().collect::<Vec<_>>())
            .field("discriminators", &self.discriminators())
            .finish()
    }
}

impl std::cmp::PartialEq for UnionType<'_> {
    fn eq(&self, other: &UnionType<'_>) -> bool {
        std::ptr::eq(self.0.registry, other.0.registry) && self.0.id == other.0.id
    }
}
impl std::cmp::Eq for UnionType<'_> {}

impl RegistryId for UnionTypeId {
    type Reader<'a> = UnionType<'a>;
}

impl IdReader for UnionType<'_> {
    type Id = UnionTypeId;
}

impl<'a> From<ReadContext<'a, UnionTypeId>> for UnionType<'a> {
    fn from(value: ReadContext<'a, UnionTypeId>) -> Self {
        Self(value)
    }
}
