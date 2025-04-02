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
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union TypeDefinition @id @meta(module: "type_definition") @variants(remove_suffix: "Definition") =
///   | ObjectDefinition
///   | InterfaceDefinition
///   | UnionDefinition
///   | EnumDefinition
///   | InputObjectDefinition
///   | ScalarDefinition
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TypeDefinitionId {
    Enum(EnumDefinitionId),
    InputObject(InputObjectDefinitionId),
    Interface(InterfaceDefinitionId),
    Object(ObjectDefinitionId),
    Scalar(ScalarDefinitionId),
    Union(UnionDefinitionId),
}

impl std::fmt::Debug for TypeDefinitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeDefinitionId::Enum(variant) => variant.fmt(f),
            TypeDefinitionId::InputObject(variant) => variant.fmt(f),
            TypeDefinitionId::Interface(variant) => variant.fmt(f),
            TypeDefinitionId::Object(variant) => variant.fmt(f),
            TypeDefinitionId::Scalar(variant) => variant.fmt(f),
            TypeDefinitionId::Union(variant) => variant.fmt(f),
        }
    }
}

impl From<EnumDefinitionId> for TypeDefinitionId {
    fn from(value: EnumDefinitionId) -> Self {
        TypeDefinitionId::Enum(value)
    }
}
impl From<InputObjectDefinitionId> for TypeDefinitionId {
    fn from(value: InputObjectDefinitionId) -> Self {
        TypeDefinitionId::InputObject(value)
    }
}
impl From<InterfaceDefinitionId> for TypeDefinitionId {
    fn from(value: InterfaceDefinitionId) -> Self {
        TypeDefinitionId::Interface(value)
    }
}
impl From<ObjectDefinitionId> for TypeDefinitionId {
    fn from(value: ObjectDefinitionId) -> Self {
        TypeDefinitionId::Object(value)
    }
}
impl From<ScalarDefinitionId> for TypeDefinitionId {
    fn from(value: ScalarDefinitionId) -> Self {
        TypeDefinitionId::Scalar(value)
    }
}
impl From<UnionDefinitionId> for TypeDefinitionId {
    fn from(value: UnionDefinitionId) -> Self {
        TypeDefinitionId::Union(value)
    }
}

impl TypeDefinitionId {
    pub fn is_enum(&self) -> bool {
        matches!(self, TypeDefinitionId::Enum(_))
    }
    pub fn as_enum(&self) -> Option<EnumDefinitionId> {
        match self {
            TypeDefinitionId::Enum(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_input_object(&self) -> bool {
        matches!(self, TypeDefinitionId::InputObject(_))
    }
    pub fn as_input_object(&self) -> Option<InputObjectDefinitionId> {
        match self {
            TypeDefinitionId::InputObject(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_interface(&self) -> bool {
        matches!(self, TypeDefinitionId::Interface(_))
    }
    pub fn as_interface(&self) -> Option<InterfaceDefinitionId> {
        match self {
            TypeDefinitionId::Interface(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_object(&self) -> bool {
        matches!(self, TypeDefinitionId::Object(_))
    }
    pub fn as_object(&self) -> Option<ObjectDefinitionId> {
        match self {
            TypeDefinitionId::Object(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_scalar(&self) -> bool {
        matches!(self, TypeDefinitionId::Scalar(_))
    }
    pub fn as_scalar(&self) -> Option<ScalarDefinitionId> {
        match self {
            TypeDefinitionId::Scalar(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_union(&self) -> bool {
        matches!(self, TypeDefinitionId::Union(_))
    }
    pub fn as_union(&self) -> Option<UnionDefinitionId> {
        match self {
            TypeDefinitionId::Union(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum TypeDefinition<'a> {
    Enum(EnumDefinition<'a>),
    InputObject(InputObjectDefinition<'a>),
    Interface(InterfaceDefinition<'a>),
    Object(ObjectDefinition<'a>),
    Scalar(ScalarDefinition<'a>),
    Union(UnionDefinition<'a>),
}

impl std::fmt::Debug for TypeDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeDefinition::Enum(variant) => variant.fmt(f),
            TypeDefinition::InputObject(variant) => variant.fmt(f),
            TypeDefinition::Interface(variant) => variant.fmt(f),
            TypeDefinition::Object(variant) => variant.fmt(f),
            TypeDefinition::Scalar(variant) => variant.fmt(f),
            TypeDefinition::Union(variant) => variant.fmt(f),
        }
    }
}

impl<'a> From<EnumDefinition<'a>> for TypeDefinition<'a> {
    fn from(item: EnumDefinition<'a>) -> Self {
        TypeDefinition::Enum(item)
    }
}
impl<'a> From<InputObjectDefinition<'a>> for TypeDefinition<'a> {
    fn from(item: InputObjectDefinition<'a>) -> Self {
        TypeDefinition::InputObject(item)
    }
}
impl<'a> From<InterfaceDefinition<'a>> for TypeDefinition<'a> {
    fn from(item: InterfaceDefinition<'a>) -> Self {
        TypeDefinition::Interface(item)
    }
}
impl<'a> From<ObjectDefinition<'a>> for TypeDefinition<'a> {
    fn from(item: ObjectDefinition<'a>) -> Self {
        TypeDefinition::Object(item)
    }
}
impl<'a> From<ScalarDefinition<'a>> for TypeDefinition<'a> {
    fn from(item: ScalarDefinition<'a>) -> Self {
        TypeDefinition::Scalar(item)
    }
}
impl<'a> From<UnionDefinition<'a>> for TypeDefinition<'a> {
    fn from(item: UnionDefinition<'a>) -> Self {
        TypeDefinition::Union(item)
    }
}

impl<'a> Walk<&'a Schema> for TypeDefinitionId {
    type Walker<'w>
        = TypeDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let schema: &'a Schema = schema.into();
        match self {
            TypeDefinitionId::Enum(id) => TypeDefinition::Enum(id.walk(schema)),
            TypeDefinitionId::InputObject(id) => TypeDefinition::InputObject(id.walk(schema)),
            TypeDefinitionId::Interface(id) => TypeDefinition::Interface(id.walk(schema)),
            TypeDefinitionId::Object(id) => TypeDefinition::Object(id.walk(schema)),
            TypeDefinitionId::Scalar(id) => TypeDefinition::Scalar(id.walk(schema)),
            TypeDefinitionId::Union(id) => TypeDefinition::Union(id.walk(schema)),
        }
    }
}

impl<'a> TypeDefinition<'a> {
    pub fn id(&self) -> TypeDefinitionId {
        match self {
            TypeDefinition::Enum(walker) => TypeDefinitionId::Enum(walker.id),
            TypeDefinition::InputObject(walker) => TypeDefinitionId::InputObject(walker.id),
            TypeDefinition::Interface(walker) => TypeDefinitionId::Interface(walker.id),
            TypeDefinition::Object(walker) => TypeDefinitionId::Object(walker.id),
            TypeDefinition::Scalar(walker) => TypeDefinitionId::Scalar(walker.id),
            TypeDefinition::Union(walker) => TypeDefinitionId::Union(walker.id),
        }
    }
    pub fn is_enum(&self) -> bool {
        matches!(self, TypeDefinition::Enum(_))
    }
    pub fn as_enum(&self) -> Option<EnumDefinition<'a>> {
        match self {
            TypeDefinition::Enum(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_input_object(&self) -> bool {
        matches!(self, TypeDefinition::InputObject(_))
    }
    pub fn as_input_object(&self) -> Option<InputObjectDefinition<'a>> {
        match self {
            TypeDefinition::InputObject(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_interface(&self) -> bool {
        matches!(self, TypeDefinition::Interface(_))
    }
    pub fn as_interface(&self) -> Option<InterfaceDefinition<'a>> {
        match self {
            TypeDefinition::Interface(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_object(&self) -> bool {
        matches!(self, TypeDefinition::Object(_))
    }
    pub fn as_object(&self) -> Option<ObjectDefinition<'a>> {
        match self {
            TypeDefinition::Object(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_scalar(&self) -> bool {
        matches!(self, TypeDefinition::Scalar(_))
    }
    pub fn as_scalar(&self) -> Option<ScalarDefinition<'a>> {
        match self {
            TypeDefinition::Scalar(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_union(&self) -> bool {
        matches!(self, TypeDefinition::Union(_))
    }
    pub fn as_union(&self) -> Option<UnionDefinition<'a>> {
        match self {
            TypeDefinition::Union(item) => Some(*item),
            _ => None,
        }
    }
}
