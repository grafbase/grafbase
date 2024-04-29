use super::prelude::*;
use super::{
    field::MetaField,
    prelude::ids::{InterfaceTypeId, MetaFieldId},
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct InterfaceTypeRecord {
    #[serde(rename = "0")]
    pub name: StringId,
    #[serde(rename = "1", skip_serializing_if = "crate::Container::is_empty", default)]
    pub fields: IdRange<MetaFieldId>,
    #[serde(rename = "2", skip_serializing_if = "Option::is_none", default)]
    pub cache_control: Option<Box<CacheControl>>,
}

#[derive(Clone, Copy)]
pub struct InterfaceType<'a>(pub(crate) ReadContext<'a, InterfaceTypeId>);

impl<'a> InterfaceType<'a> {
    pub fn name(&self) -> &'a str {
        let registry = &self.0.registry;
        registry.lookup(registry.lookup(self.0.id).name)
    }
    pub fn fields(&self) -> Iter<'a, MetaField<'a>> {
        let registry = self.0.registry;
        Iter::new(registry.lookup(self.0.id).fields, registry)
    }
    pub fn cache_control(&self) -> Option<&'a CacheControl> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).cache_control.as_deref()
    }
}

impl fmt::Debug for InterfaceType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InterfaceType")
            .field("name", &self.name())
            .field("fields", &self.fields().collect::<Vec<_>>())
            .field("cache_control", &self.cache_control())
            .finish()
    }
}

impl std::cmp::PartialEq for InterfaceType<'_> {
    fn eq(&self, other: &InterfaceType<'_>) -> bool {
        std::ptr::eq(self.0.registry, other.0.registry) && self.0.id == other.0.id
    }
}
impl std::cmp::Eq for InterfaceType<'_> {}

impl RegistryId for InterfaceTypeId {
    type Reader<'a> = InterfaceType<'a>;
}

impl IdReader for InterfaceType<'_> {
    type Id = InterfaceTypeId;
}

impl<'a> From<ReadContext<'a, InterfaceTypeId>> for InterfaceType<'a> {
    fn from(value: ReadContext<'a, InterfaceTypeId>) -> Self {
        Self(value)
    }
}