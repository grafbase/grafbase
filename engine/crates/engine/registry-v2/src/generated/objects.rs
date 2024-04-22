use super::prelude::*;
use super::{
    field::MetaField,
    prelude::ids::{MetaFieldId, ObjectTypeId},
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct ObjectTypeRecord {
    #[serde(rename = "0")]
    pub name: StringId,
    #[serde(rename = "1", skip_serializing_if = "Option::is_none", default)]
    pub description: Option<StringId>,
    #[serde(rename = "2", skip_serializing_if = "crate::Container::is_empty", default)]
    pub fields: IdRange<MetaFieldId>,
    #[serde(rename = "3", skip_serializing_if = "Option::is_none", default)]
    pub cache_control: Option<Box<CacheControl>>,
    #[serde(rename = "4", skip_serializing_if = "crate::is_false", default)]
    pub external: bool,
    #[serde(rename = "5", skip_serializing_if = "crate::is_false", default)]
    pub shareable: bool,
}

#[derive(Clone, Copy)]
pub struct ObjectType<'a>(pub(crate) ReadContext<'a, ObjectTypeId>);

impl<'a> ObjectType<'a> {
    pub fn name(&self) -> &'a str {
        let registry = &self.0.registry;
        registry.lookup(registry.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).description.map(|id| registry.lookup(id))
    }
    pub fn fields(&self) -> Iter<'a, MetaField<'a>> {
        let registry = self.0.registry;
        Iter::new(registry.lookup(self.0.id).fields, registry)
    }
    pub fn cache_control(&self) -> Option<&'a CacheControl> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).cache_control.as_deref()
    }
    pub fn external(&self) -> bool {
        let registry = self.0.registry;
        registry.lookup(self.0.id).external
    }
    pub fn shareable(&self) -> bool {
        let registry = self.0.registry;
        registry.lookup(self.0.id).shareable
    }
}

impl fmt::Debug for ObjectType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectType")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("fields", &self.fields().collect::<Vec<_>>())
            .field("cache_control", &self.cache_control())
            .field("external", &self.external())
            .field("shareable", &self.shareable())
            .finish()
    }
}

impl std::cmp::PartialEq for ObjectType<'_> {
    fn eq(&self, other: &ObjectType<'_>) -> bool {
        std::ptr::eq(self.0.registry, other.0.registry) && self.0.id == other.0.id
    }
}
impl std::cmp::Eq for ObjectType<'_> {}

impl RegistryId for ObjectTypeId {
    type Reader<'a> = ObjectType<'a>;
}

impl IdReader for ObjectType<'_> {
    type Id = ObjectTypeId;
}

impl<'a> From<ReadContext<'a, ObjectTypeId>> for ObjectType<'a> {
    fn from(value: ReadContext<'a, ObjectTypeId>) -> Self {
        Self(value)
    }
}
