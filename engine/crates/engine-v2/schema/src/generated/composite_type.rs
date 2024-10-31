//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{
        InterfaceDefinition, InterfaceDefinitionId, ObjectDefinition, ObjectDefinitionId, UnionDefinition,
        UnionDefinitionId,
    },
    prelude::*,
};
use walker::Walk;

/// Name previously used by the GraphQL spec to describe this union.
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// union CompositeType @id @meta(module: "composite_type") @variants(remove_suffix: "Definition") =
///   | ObjectDefinition
///   | InterfaceDefinition
///   | UnionDefinition
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompositeTypeId {
    Interface(InterfaceDefinitionId),
    Object(ObjectDefinitionId),
    Union(UnionDefinitionId),
}

impl std::fmt::Debug for CompositeTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompositeTypeId::Interface(variant) => variant.fmt(f),
            CompositeTypeId::Object(variant) => variant.fmt(f),
            CompositeTypeId::Union(variant) => variant.fmt(f),
        }
    }
}

impl From<InterfaceDefinitionId> for CompositeTypeId {
    fn from(value: InterfaceDefinitionId) -> Self {
        CompositeTypeId::Interface(value)
    }
}
impl From<ObjectDefinitionId> for CompositeTypeId {
    fn from(value: ObjectDefinitionId) -> Self {
        CompositeTypeId::Object(value)
    }
}
impl From<UnionDefinitionId> for CompositeTypeId {
    fn from(value: UnionDefinitionId) -> Self {
        CompositeTypeId::Union(value)
    }
}

/// Name previously used by the GraphQL spec to describe this union.
#[derive(Clone, Copy)]
pub enum CompositeType<'a> {
    Interface(InterfaceDefinition<'a>),
    Object(ObjectDefinition<'a>),
    Union(UnionDefinition<'a>),
}

impl std::fmt::Debug for CompositeType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompositeType::Interface(variant) => variant.fmt(f),
            CompositeType::Object(variant) => variant.fmt(f),
            CompositeType::Union(variant) => variant.fmt(f),
        }
    }
}

impl Walk<Schema> for CompositeTypeId {
    type Walker<'a> = CompositeType<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        match self {
            CompositeTypeId::Interface(id) => CompositeType::Interface(id.walk(schema)),
            CompositeTypeId::Object(id) => CompositeType::Object(id.walk(schema)),
            CompositeTypeId::Union(id) => CompositeType::Union(id.walk(schema)),
        }
    }
}

impl CompositeType<'_> {
    pub fn id(&self) -> CompositeTypeId {
        match self {
            CompositeType::Interface(walker) => CompositeTypeId::Interface(walker.id),
            CompositeType::Object(walker) => CompositeTypeId::Object(walker.id),
            CompositeType::Union(walker) => CompositeTypeId::Union(walker.id),
        }
    }
}
