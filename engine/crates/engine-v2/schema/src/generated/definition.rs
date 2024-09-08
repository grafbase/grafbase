//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{
        EnumDefinition, EnumDefinitionId, InputObjectDefinition, InputObjectDefinitionId, InterfaceDefinition,
        InterfaceDefinitionId, ObjectDefinition, ObjectDefinitionId, ScalarDefinition, ScalarDefinitionId,
        UnionDefinition, UnionDefinitionId,
    },
    prelude::*,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union Definition @id @meta(module: "definition") @variants(remove_suffix: true) =
///   | ObjectDefinition
///   | InterfaceDefinition
///   | UnionDefinition
///   | EnumDefinition
///   | InputObjectDefinition
///   | ScalarDefinition
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DefinitionId {
    Enum(EnumDefinitionId),
    InputObject(InputObjectDefinitionId),
    Interface(InterfaceDefinitionId),
    Object(ObjectDefinitionId),
    Scalar(ScalarDefinitionId),
    Union(UnionDefinitionId),
}

impl std::fmt::Debug for DefinitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefinitionId::Enum(variant) => variant.fmt(f),
            DefinitionId::InputObject(variant) => variant.fmt(f),
            DefinitionId::Interface(variant) => variant.fmt(f),
            DefinitionId::Object(variant) => variant.fmt(f),
            DefinitionId::Scalar(variant) => variant.fmt(f),
            DefinitionId::Union(variant) => variant.fmt(f),
        }
    }
}

impl From<EnumDefinitionId> for DefinitionId {
    fn from(value: EnumDefinitionId) -> Self {
        DefinitionId::Enum(value)
    }
}
impl From<InputObjectDefinitionId> for DefinitionId {
    fn from(value: InputObjectDefinitionId) -> Self {
        DefinitionId::InputObject(value)
    }
}
impl From<InterfaceDefinitionId> for DefinitionId {
    fn from(value: InterfaceDefinitionId) -> Self {
        DefinitionId::Interface(value)
    }
}
impl From<ObjectDefinitionId> for DefinitionId {
    fn from(value: ObjectDefinitionId) -> Self {
        DefinitionId::Object(value)
    }
}
impl From<ScalarDefinitionId> for DefinitionId {
    fn from(value: ScalarDefinitionId) -> Self {
        DefinitionId::Scalar(value)
    }
}
impl From<UnionDefinitionId> for DefinitionId {
    fn from(value: UnionDefinitionId) -> Self {
        DefinitionId::Union(value)
    }
}

#[derive(Clone, Copy)]
pub enum Definition<'a> {
    Enum(EnumDefinition<'a>),
    InputObject(InputObjectDefinition<'a>),
    Interface(InterfaceDefinition<'a>),
    Object(ObjectDefinition<'a>),
    Scalar(ScalarDefinition<'a>),
    Union(UnionDefinition<'a>),
}

impl std::fmt::Debug for Definition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Definition::Enum(variant) => variant.fmt(f),
            Definition::InputObject(variant) => variant.fmt(f),
            Definition::Interface(variant) => variant.fmt(f),
            Definition::Object(variant) => variant.fmt(f),
            Definition::Scalar(variant) => variant.fmt(f),
            Definition::Union(variant) => variant.fmt(f),
        }
    }
}

impl Walk<Schema> for DefinitionId {
    type Walker<'a> = Definition<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        match self {
            DefinitionId::Enum(id) => Definition::Enum(id.walk(schema)),
            DefinitionId::InputObject(id) => Definition::InputObject(id.walk(schema)),
            DefinitionId::Interface(id) => Definition::Interface(id.walk(schema)),
            DefinitionId::Object(id) => Definition::Object(id.walk(schema)),
            DefinitionId::Scalar(id) => Definition::Scalar(id.walk(schema)),
            DefinitionId::Union(id) => Definition::Union(id.walk(schema)),
        }
    }
}

impl Definition<'_> {
    pub fn id(&self) -> DefinitionId {
        match self {
            Definition::Enum(walker) => DefinitionId::Enum(walker.id),
            Definition::InputObject(walker) => DefinitionId::InputObject(walker.id),
            Definition::Interface(walker) => DefinitionId::Interface(walker.id),
            Definition::Object(walker) => DefinitionId::Object(walker.id),
            Definition::Scalar(walker) => DefinitionId::Scalar(walker.id),
            Definition::Union(walker) => DefinitionId::Union(walker.id),
        }
    }
}
