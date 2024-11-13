//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
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
///     ObjectDefinition
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

impl DefinitionId {
    pub fn is_enum(&self) -> bool {
        matches!(self, DefinitionId::Enum(_))
    }
    pub fn as_enum(&self) -> Option<EnumDefinitionId> {
        match self {
            DefinitionId::Enum(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_input_object(&self) -> bool {
        matches!(self, DefinitionId::InputObject(_))
    }
    pub fn as_input_object(&self) -> Option<InputObjectDefinitionId> {
        match self {
            DefinitionId::InputObject(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_interface(&self) -> bool {
        matches!(self, DefinitionId::Interface(_))
    }
    pub fn as_interface(&self) -> Option<InterfaceDefinitionId> {
        match self {
            DefinitionId::Interface(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_object(&self) -> bool {
        matches!(self, DefinitionId::Object(_))
    }
    pub fn as_object(&self) -> Option<ObjectDefinitionId> {
        match self {
            DefinitionId::Object(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_scalar(&self) -> bool {
        matches!(self, DefinitionId::Scalar(_))
    }
    pub fn as_scalar(&self) -> Option<ScalarDefinitionId> {
        match self {
            DefinitionId::Scalar(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_union(&self) -> bool {
        matches!(self, DefinitionId::Union(_))
    }
    pub fn as_union(&self) -> Option<UnionDefinitionId> {
        match self {
            DefinitionId::Union(id) => Some(*id),
            _ => None,
        }
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

impl<'a> From<EnumDefinition<'a>> for Definition<'a> {
    fn from(item: EnumDefinition<'a>) -> Self {
        Definition::Enum(item)
    }
}
impl<'a> From<InputObjectDefinition<'a>> for Definition<'a> {
    fn from(item: InputObjectDefinition<'a>) -> Self {
        Definition::InputObject(item)
    }
}
impl<'a> From<InterfaceDefinition<'a>> for Definition<'a> {
    fn from(item: InterfaceDefinition<'a>) -> Self {
        Definition::Interface(item)
    }
}
impl<'a> From<ObjectDefinition<'a>> for Definition<'a> {
    fn from(item: ObjectDefinition<'a>) -> Self {
        Definition::Object(item)
    }
}
impl<'a> From<ScalarDefinition<'a>> for Definition<'a> {
    fn from(item: ScalarDefinition<'a>) -> Self {
        Definition::Scalar(item)
    }
}
impl<'a> From<UnionDefinition<'a>> for Definition<'a> {
    fn from(item: UnionDefinition<'a>) -> Self {
        Definition::Union(item)
    }
}

impl<'a> Walk<&'a Schema> for DefinitionId {
    type Walker<'w> = Definition<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let schema: &'a Schema = schema.into();
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

impl<'a> Definition<'a> {
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
    pub fn is_enum(&self) -> bool {
        matches!(self, Definition::Enum(_))
    }
    pub fn as_enum(&self) -> Option<EnumDefinition<'a>> {
        match self {
            Definition::Enum(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_input_object(&self) -> bool {
        matches!(self, Definition::InputObject(_))
    }
    pub fn as_input_object(&self) -> Option<InputObjectDefinition<'a>> {
        match self {
            Definition::InputObject(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_interface(&self) -> bool {
        matches!(self, Definition::Interface(_))
    }
    pub fn as_interface(&self) -> Option<InterfaceDefinition<'a>> {
        match self {
            Definition::Interface(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_object(&self) -> bool {
        matches!(self, Definition::Object(_))
    }
    pub fn as_object(&self) -> Option<ObjectDefinition<'a>> {
        match self {
            Definition::Object(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_scalar(&self) -> bool {
        matches!(self, Definition::Scalar(_))
    }
    pub fn as_scalar(&self) -> Option<ScalarDefinition<'a>> {
        match self {
            Definition::Scalar(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_union(&self) -> bool {
        matches!(self, Definition::Union(_))
    }
    pub fn as_union(&self) -> Option<UnionDefinition<'a>> {
        match self {
            Definition::Union(item) => Some(*item),
            _ => None,
        }
    }
}
