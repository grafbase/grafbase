use super::prelude::*;
use super::{
    enums::EnumType,
    inputs::InputObjectType,
    interface::InterfaceType,
    objects::ObjectType,
    prelude::ids::{
        EnumTypeId, InputObjectTypeId, InterfaceTypeId, MetaTypeId, ObjectTypeId, ScalarTypeId, UnionTypeId,
    },
    scalar::ScalarType,
    union::UnionType,
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
    Union(UnionTypeId),
    #[serde(rename = "3")]
    Enum(EnumTypeId),
    #[serde(rename = "4")]
    InputObject(InputObjectTypeId),
    #[serde(rename = "5")]
    Scalar(ScalarTypeId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetaType<'a> {
    Object(ObjectType<'a>),
    Interface(InterfaceType<'a>),
    Union(UnionType<'a>),
    Enum(EnumType<'a>),
    InputObject(InputObjectType<'a>),
    Scalar(ScalarType<'a>),
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
            MetaTypeRecord::Union(id) => MetaType::Union(value.registry.read(*id)),
            MetaTypeRecord::Enum(id) => MetaType::Enum(value.registry.read(*id)),
            MetaTypeRecord::InputObject(id) => MetaType::InputObject(value.registry.read(*id)),
            MetaTypeRecord::Scalar(id) => MetaType::Scalar(value.registry.read(*id)),
        }
    }
}
