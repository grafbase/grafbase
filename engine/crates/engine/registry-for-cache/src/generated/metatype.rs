use super::prelude::*;
use super::{
    interface::InterfaceType,
    objects::ObjectType,
    prelude::ids::{InterfaceTypeId, MetaTypeId, ObjectTypeId},
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

#[derive(serde :: Serialize, serde :: Deserialize)]
pub enum MetaTypeRecord {
    #[serde(rename = "0")]
    Object(ObjectTypeId),
    #[serde(rename = "1")]
    Interface(InterfaceTypeId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaType<'a> {
    Object(ObjectType<'a>),
    Interface(InterfaceType<'a>),
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
        }
    }
}