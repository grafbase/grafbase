//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{
        InterfaceDefinition, InterfaceDefinitionId, ObjectDefinition, ObjectDefinitionId, UnionDefinition,
        UnionDefinitionId,
    },
    prelude::*,
};
use walker::Walk;

/// Composite type is the term previously used by the GraphQL spec to describe this union.
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

impl CompositeTypeId {
    pub fn is_interface(&self) -> bool {
        matches!(self, CompositeTypeId::Interface(_))
    }
    pub fn as_interface(&self) -> Option<InterfaceDefinitionId> {
        match self {
            CompositeTypeId::Interface(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_object(&self) -> bool {
        matches!(self, CompositeTypeId::Object(_))
    }
    pub fn as_object(&self) -> Option<ObjectDefinitionId> {
        match self {
            CompositeTypeId::Object(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_union(&self) -> bool {
        matches!(self, CompositeTypeId::Union(_))
    }
    pub fn as_union(&self) -> Option<UnionDefinitionId> {
        match self {
            CompositeTypeId::Union(id) => Some(*id),
            _ => None,
        }
    }
}

/// Composite type is the term previously used by the GraphQL spec to describe this union.
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

impl<'a> From<InterfaceDefinition<'a>> for CompositeType<'a> {
    fn from(item: InterfaceDefinition<'a>) -> Self {
        CompositeType::Interface(item)
    }
}
impl<'a> From<ObjectDefinition<'a>> for CompositeType<'a> {
    fn from(item: ObjectDefinition<'a>) -> Self {
        CompositeType::Object(item)
    }
}
impl<'a> From<UnionDefinition<'a>> for CompositeType<'a> {
    fn from(item: UnionDefinition<'a>) -> Self {
        CompositeType::Union(item)
    }
}

impl<'a> Walk<&'a Schema> for CompositeTypeId {
    type Walker<'w> = CompositeType<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let schema: &'a Schema = schema.into();
        match self {
            CompositeTypeId::Interface(id) => CompositeType::Interface(id.walk(schema)),
            CompositeTypeId::Object(id) => CompositeType::Object(id.walk(schema)),
            CompositeTypeId::Union(id) => CompositeType::Union(id.walk(schema)),
        }
    }
}

impl<'a> CompositeType<'a> {
    pub fn id(&self) -> CompositeTypeId {
        match self {
            CompositeType::Interface(walker) => CompositeTypeId::Interface(walker.id),
            CompositeType::Object(walker) => CompositeTypeId::Object(walker.id),
            CompositeType::Union(walker) => CompositeTypeId::Union(walker.id),
        }
    }
    pub fn is_interface(&self) -> bool {
        matches!(self, CompositeType::Interface(_))
    }
    pub fn as_interface(&self) -> Option<InterfaceDefinition<'a>> {
        match self {
            CompositeType::Interface(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_object(&self) -> bool {
        matches!(self, CompositeType::Object(_))
    }
    pub fn as_object(&self) -> Option<ObjectDefinition<'a>> {
        match self {
            CompositeType::Object(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_union(&self) -> bool {
        matches!(self, CompositeType::Union(_))
    }
    pub fn as_union(&self) -> Option<UnionDefinition<'a>> {
        match self {
            CompositeType::Union(item) => Some(*item),
            _ => None,
        }
    }
}
