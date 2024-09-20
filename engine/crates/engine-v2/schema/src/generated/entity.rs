//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
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

impl Walk<Schema> for EntityDefinitionId {
    type Walker<'a> = EntityDefinition<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        match self {
            EntityDefinitionId::Interface(id) => EntityDefinition::Interface(id.walk(schema)),
            EntityDefinitionId::Object(id) => EntityDefinition::Object(id.walk(schema)),
        }
    }
}

impl EntityDefinition<'_> {
    pub fn id(&self) -> EntityDefinitionId {
        match self {
            EntityDefinition::Interface(walker) => EntityDefinitionId::Interface(walker.id),
            EntityDefinition::Object(walker) => EntityDefinitionId::Object(walker.id),
        }
    }
}
