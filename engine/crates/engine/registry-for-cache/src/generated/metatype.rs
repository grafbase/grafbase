use super::prelude::*;
use super::{
    interface::InterfaceType,
    objects::ObjectType,
    others::OtherType,
    prelude::ids::{InterfaceTypeId, MetaTypeId, ObjectTypeId, OtherTypeId},
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

#[derive(serde :: Serialize, serde :: Deserialize)]
pub enum MetaTypeRecord {
    #[serde(rename = "0")]
    Object(ObjectTypeId),
    #[serde(rename = "1")]
    Interface(InterfaceTypeId),
    #[serde(rename = "2")]
    Other(OtherTypeId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaType<'a> {
    Object(ObjectType<'a>),
    Interface(InterfaceType<'a>),
    Other(OtherType<'a>),
}

impl RegistryId for MetaTypeId {
    type Reader<'a> = MetaType<'a>;
}

impl IdReader for MetaType<'_> {
    type Id = MetaTypeId;
}

impl<'a> From<ReadContext<'a, MetaTypeId>> for MetaType<'a> {
    fn from(value: ReadContext<'a, MetaTypeId>) -> Self {
        match value.registry.lookup(value.id) {
            MetaTypeRecord::Object(id) => MetaType::Object(value.registry.read(*id)),
            MetaTypeRecord::Interface(id) => MetaType::Interface(value.registry.read(*id)),
            MetaTypeRecord::Other(id) => MetaType::Other(value.registry.read(*id)),
        }
    }
}
