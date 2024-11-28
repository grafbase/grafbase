//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{InterfaceDefinition, InterfaceDefinitionId, ObjectDefinition, ObjectDefinitionId},
    prelude::*,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union EntityDefinition @id @meta(module: "entity") @variants(remove_suffix: "Definition") =
///   | ObjectDefinition
///   | InterfaceDefinition
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityDefinitionId {
    Interface(InterfaceDefinitionId),
    Object(ObjectDefinitionId),
}

impl std::fmt::Debug for EntityDefinitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityDefinitionId::Interface(variant) => variant.fmt(f),
            EntityDefinitionId::Object(variant) => variant.fmt(f),
        }
    }
}

impl From<InterfaceDefinitionId> for EntityDefinitionId {
    fn from(value: InterfaceDefinitionId) -> Self {
        EntityDefinitionId::Interface(value)
    }
}
impl From<ObjectDefinitionId> for EntityDefinitionId {
    fn from(value: ObjectDefinitionId) -> Self {
        EntityDefinitionId::Object(value)
    }
}

impl EntityDefinitionId {
    pub fn is_interface(&self) -> bool {
        matches!(self, EntityDefinitionId::Interface(_))
    }
    pub fn as_interface(&self) -> Option<InterfaceDefinitionId> {
        match self {
            EntityDefinitionId::Interface(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_object(&self) -> bool {
        matches!(self, EntityDefinitionId::Object(_))
    }
    pub fn as_object(&self) -> Option<ObjectDefinitionId> {
        match self {
            EntityDefinitionId::Object(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum EntityDefinition<'a> {
    Interface(InterfaceDefinition<'a>),
    Object(ObjectDefinition<'a>),
}

impl std::fmt::Debug for EntityDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityDefinition::Interface(variant) => variant.fmt(f),
            EntityDefinition::Object(variant) => variant.fmt(f),
        }
    }
}

impl<'a> From<InterfaceDefinition<'a>> for EntityDefinition<'a> {
    fn from(item: InterfaceDefinition<'a>) -> Self {
        EntityDefinition::Interface(item)
    }
}
impl<'a> From<ObjectDefinition<'a>> for EntityDefinition<'a> {
    fn from(item: ObjectDefinition<'a>) -> Self {
        EntityDefinition::Object(item)
    }
}

impl<'a> Walk<&'a Schema> for EntityDefinitionId {
    type Walker<'w>
        = EntityDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let schema: &'a Schema = schema.into();
        match self {
            EntityDefinitionId::Interface(id) => EntityDefinition::Interface(id.walk(schema)),
            EntityDefinitionId::Object(id) => EntityDefinition::Object(id.walk(schema)),
        }
    }
}

impl<'a> EntityDefinition<'a> {
    pub fn id(&self) -> EntityDefinitionId {
        match self {
            EntityDefinition::Interface(walker) => EntityDefinitionId::Interface(walker.id),
            EntityDefinition::Object(walker) => EntityDefinitionId::Object(walker.id),
        }
    }
    pub fn is_interface(&self) -> bool {
        matches!(self, EntityDefinition::Interface(_))
    }
    pub fn as_interface(&self) -> Option<InterfaceDefinition<'a>> {
        match self {
            EntityDefinition::Interface(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_object(&self) -> bool {
        matches!(self, EntityDefinition::Object(_))
    }
    pub fn as_object(&self) -> Option<ObjectDefinition<'a>> {
        match self {
            EntityDefinition::Object(item) => Some(*item),
            _ => None,
        }
    }
}
