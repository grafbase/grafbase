use super::prelude::ids::{EnumTypeId, MetaEnumValueId};
use super::prelude::*;
#[allow(unused_imports)]
use std::fmt::{self, Write};

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct EnumTypeRecord {
    #[serde(rename = "0")]
    pub name: StringId,
    #[serde(rename = "1", skip_serializing_if = "Option::is_none", default)]
    pub description: Option<StringId>,
    #[serde(rename = "2", skip_serializing_if = "crate::Container::is_empty", default)]
    pub values: IdRange<MetaEnumValueId>,
}

#[derive(Clone, Copy)]
pub struct EnumType<'a>(pub(crate) ReadContext<'a, EnumTypeId>);

impl<'a> EnumType<'a> {
    pub fn name(&self) -> &'a str {
        let registry = &self.0.registry;
        registry.lookup(registry.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).description.map(|id| registry.lookup(id))
    }
    pub fn values(&self) -> Iter<'a, MetaEnumValue<'a>> {
        let registry = self.0.registry;
        Iter::new(registry.lookup(self.0.id).values, registry)
    }
}

impl fmt::Debug for EnumType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnumType")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("values", &self.values().collect::<Vec<_>>())
            .finish()
    }
}

impl std::cmp::PartialEq for EnumType<'_> {
    fn eq(&self, other: &EnumType<'_>) -> bool {
        std::ptr::eq(self.0.registry, other.0.registry) && self.0.id == other.0.id
    }
}
impl std::cmp::Eq for EnumType<'_> {}

impl RegistryId for EnumTypeId {
    type Reader<'a> = EnumType<'a>;
}

impl IdReader for EnumType<'_> {
    type Id = EnumTypeId;
}

impl<'a> From<ReadContext<'a, EnumTypeId>> for EnumType<'a> {
    fn from(value: ReadContext<'a, EnumTypeId>) -> Self {
        Self(value)
    }
}

#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct MetaEnumValueRecord {
    #[serde(rename = "0")]
    pub name: StringId,
    #[serde(rename = "1", skip_serializing_if = "Option::is_none", default)]
    pub description: Option<StringId>,
    #[serde(rename = "2", skip_serializing_if = "Option::is_none", default)]
    pub deprecation: Option<Box<Deprecation>>,
    #[serde(rename = "3", skip_serializing_if = "Option::is_none", default)]
    pub value: Option<StringId>,
}

#[derive(Clone, Copy)]
pub struct MetaEnumValue<'a>(pub(crate) ReadContext<'a, MetaEnumValueId>);

impl<'a> MetaEnumValue<'a> {
    pub fn name(&self) -> &'a str {
        let registry = &self.0.registry;
        registry.lookup(registry.lookup(self.0.id).name)
    }
    pub fn description(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).description.map(|id| registry.lookup(id))
    }
    pub fn deprecation(&self) -> Option<&'a Deprecation> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).deprecation.as_deref()
    }
    pub fn value(&self) -> Option<&'a str> {
        let registry = self.0.registry;
        registry.lookup(self.0.id).value.map(|id| registry.lookup(id))
    }
}

impl fmt::Debug for MetaEnumValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MetaEnumValue")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("deprecation", &self.deprecation())
            .field("value", &self.value())
            .finish()
    }
}

impl RegistryId for MetaEnumValueId {
    type Reader<'a> = MetaEnumValue<'a>;
}

impl IdReader for MetaEnumValue<'_> {
    type Id = MetaEnumValueId;
}

impl<'a> From<ReadContext<'a, MetaEnumValueId>> for MetaEnumValue<'a> {
    fn from(value: ReadContext<'a, MetaEnumValueId>) -> Self {
        Self(value)
    }
}
