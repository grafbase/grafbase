//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{
        EnumDefinition, EnumDefinitionId, EnumValue, EnumValueId, FieldDefinition, FieldDefinitionId,
        InputObjectDefinition, InputObjectDefinitionId, InputValueDefinition, InputValueDefinitionId,
        InterfaceDefinition, InterfaceDefinitionId, ObjectDefinition, ObjectDefinitionId, ScalarDefinition,
        ScalarDefinitionId, UnionDefinition, UnionDefinitionId,
    },
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union DirectiveSite @id @meta(module: "directive_site") @variants(remove_suffix: "Definition") =
///   | ObjectDefinition
///   | InterfaceDefinition
///   | UnionDefinition
///   | EnumDefinition
///   | InputObjectDefinition
///   | ScalarDefinition
///   | FieldDefinition
///   | InputValueDefinition
///   | EnumValue
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DirectiveSiteId {
    Enum(EnumDefinitionId),
    EnumValue(EnumValueId),
    Field(FieldDefinitionId),
    InputObject(InputObjectDefinitionId),
    InputValue(InputValueDefinitionId),
    Interface(InterfaceDefinitionId),
    Object(ObjectDefinitionId),
    Scalar(ScalarDefinitionId),
    Union(UnionDefinitionId),
}

impl std::fmt::Debug for DirectiveSiteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectiveSiteId::Enum(variant) => variant.fmt(f),
            DirectiveSiteId::EnumValue(variant) => variant.fmt(f),
            DirectiveSiteId::Field(variant) => variant.fmt(f),
            DirectiveSiteId::InputObject(variant) => variant.fmt(f),
            DirectiveSiteId::InputValue(variant) => variant.fmt(f),
            DirectiveSiteId::Interface(variant) => variant.fmt(f),
            DirectiveSiteId::Object(variant) => variant.fmt(f),
            DirectiveSiteId::Scalar(variant) => variant.fmt(f),
            DirectiveSiteId::Union(variant) => variant.fmt(f),
        }
    }
}

impl From<EnumDefinitionId> for DirectiveSiteId {
    fn from(value: EnumDefinitionId) -> Self {
        DirectiveSiteId::Enum(value)
    }
}
impl From<EnumValueId> for DirectiveSiteId {
    fn from(value: EnumValueId) -> Self {
        DirectiveSiteId::EnumValue(value)
    }
}
impl From<FieldDefinitionId> for DirectiveSiteId {
    fn from(value: FieldDefinitionId) -> Self {
        DirectiveSiteId::Field(value)
    }
}
impl From<InputObjectDefinitionId> for DirectiveSiteId {
    fn from(value: InputObjectDefinitionId) -> Self {
        DirectiveSiteId::InputObject(value)
    }
}
impl From<InputValueDefinitionId> for DirectiveSiteId {
    fn from(value: InputValueDefinitionId) -> Self {
        DirectiveSiteId::InputValue(value)
    }
}
impl From<InterfaceDefinitionId> for DirectiveSiteId {
    fn from(value: InterfaceDefinitionId) -> Self {
        DirectiveSiteId::Interface(value)
    }
}
impl From<ObjectDefinitionId> for DirectiveSiteId {
    fn from(value: ObjectDefinitionId) -> Self {
        DirectiveSiteId::Object(value)
    }
}
impl From<ScalarDefinitionId> for DirectiveSiteId {
    fn from(value: ScalarDefinitionId) -> Self {
        DirectiveSiteId::Scalar(value)
    }
}
impl From<UnionDefinitionId> for DirectiveSiteId {
    fn from(value: UnionDefinitionId) -> Self {
        DirectiveSiteId::Union(value)
    }
}

impl DirectiveSiteId {
    pub fn is_enum(&self) -> bool {
        matches!(self, DirectiveSiteId::Enum(_))
    }
    pub fn as_enum(&self) -> Option<EnumDefinitionId> {
        match self {
            DirectiveSiteId::Enum(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_enum_value(&self) -> bool {
        matches!(self, DirectiveSiteId::EnumValue(_))
    }
    pub fn as_enum_value(&self) -> Option<EnumValueId> {
        match self {
            DirectiveSiteId::EnumValue(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_field(&self) -> bool {
        matches!(self, DirectiveSiteId::Field(_))
    }
    pub fn as_field(&self) -> Option<FieldDefinitionId> {
        match self {
            DirectiveSiteId::Field(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_input_object(&self) -> bool {
        matches!(self, DirectiveSiteId::InputObject(_))
    }
    pub fn as_input_object(&self) -> Option<InputObjectDefinitionId> {
        match self {
            DirectiveSiteId::InputObject(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_input_value(&self) -> bool {
        matches!(self, DirectiveSiteId::InputValue(_))
    }
    pub fn as_input_value(&self) -> Option<InputValueDefinitionId> {
        match self {
            DirectiveSiteId::InputValue(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_interface(&self) -> bool {
        matches!(self, DirectiveSiteId::Interface(_))
    }
    pub fn as_interface(&self) -> Option<InterfaceDefinitionId> {
        match self {
            DirectiveSiteId::Interface(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_object(&self) -> bool {
        matches!(self, DirectiveSiteId::Object(_))
    }
    pub fn as_object(&self) -> Option<ObjectDefinitionId> {
        match self {
            DirectiveSiteId::Object(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_scalar(&self) -> bool {
        matches!(self, DirectiveSiteId::Scalar(_))
    }
    pub fn as_scalar(&self) -> Option<ScalarDefinitionId> {
        match self {
            DirectiveSiteId::Scalar(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_union(&self) -> bool {
        matches!(self, DirectiveSiteId::Union(_))
    }
    pub fn as_union(&self) -> Option<UnionDefinitionId> {
        match self {
            DirectiveSiteId::Union(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum DirectiveSite<'a> {
    Enum(EnumDefinition<'a>),
    EnumValue(EnumValue<'a>),
    Field(FieldDefinition<'a>),
    InputObject(InputObjectDefinition<'a>),
    InputValue(InputValueDefinition<'a>),
    Interface(InterfaceDefinition<'a>),
    Object(ObjectDefinition<'a>),
    Scalar(ScalarDefinition<'a>),
    Union(UnionDefinition<'a>),
}

impl std::fmt::Debug for DirectiveSite<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectiveSite::Enum(variant) => variant.fmt(f),
            DirectiveSite::EnumValue(variant) => variant.fmt(f),
            DirectiveSite::Field(variant) => variant.fmt(f),
            DirectiveSite::InputObject(variant) => variant.fmt(f),
            DirectiveSite::InputValue(variant) => variant.fmt(f),
            DirectiveSite::Interface(variant) => variant.fmt(f),
            DirectiveSite::Object(variant) => variant.fmt(f),
            DirectiveSite::Scalar(variant) => variant.fmt(f),
            DirectiveSite::Union(variant) => variant.fmt(f),
        }
    }
}

impl<'a> From<EnumDefinition<'a>> for DirectiveSite<'a> {
    fn from(item: EnumDefinition<'a>) -> Self {
        DirectiveSite::Enum(item)
    }
}
impl<'a> From<EnumValue<'a>> for DirectiveSite<'a> {
    fn from(item: EnumValue<'a>) -> Self {
        DirectiveSite::EnumValue(item)
    }
}
impl<'a> From<FieldDefinition<'a>> for DirectiveSite<'a> {
    fn from(item: FieldDefinition<'a>) -> Self {
        DirectiveSite::Field(item)
    }
}
impl<'a> From<InputObjectDefinition<'a>> for DirectiveSite<'a> {
    fn from(item: InputObjectDefinition<'a>) -> Self {
        DirectiveSite::InputObject(item)
    }
}
impl<'a> From<InputValueDefinition<'a>> for DirectiveSite<'a> {
    fn from(item: InputValueDefinition<'a>) -> Self {
        DirectiveSite::InputValue(item)
    }
}
impl<'a> From<InterfaceDefinition<'a>> for DirectiveSite<'a> {
    fn from(item: InterfaceDefinition<'a>) -> Self {
        DirectiveSite::Interface(item)
    }
}
impl<'a> From<ObjectDefinition<'a>> for DirectiveSite<'a> {
    fn from(item: ObjectDefinition<'a>) -> Self {
        DirectiveSite::Object(item)
    }
}
impl<'a> From<ScalarDefinition<'a>> for DirectiveSite<'a> {
    fn from(item: ScalarDefinition<'a>) -> Self {
        DirectiveSite::Scalar(item)
    }
}
impl<'a> From<UnionDefinition<'a>> for DirectiveSite<'a> {
    fn from(item: UnionDefinition<'a>) -> Self {
        DirectiveSite::Union(item)
    }
}

impl<'a> Walk<&'a Schema> for DirectiveSiteId {
    type Walker<'w>
        = DirectiveSite<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let schema: &'a Schema = schema.into();
        match self {
            DirectiveSiteId::Enum(id) => DirectiveSite::Enum(id.walk(schema)),
            DirectiveSiteId::EnumValue(id) => DirectiveSite::EnumValue(id.walk(schema)),
            DirectiveSiteId::Field(id) => DirectiveSite::Field(id.walk(schema)),
            DirectiveSiteId::InputObject(id) => DirectiveSite::InputObject(id.walk(schema)),
            DirectiveSiteId::InputValue(id) => DirectiveSite::InputValue(id.walk(schema)),
            DirectiveSiteId::Interface(id) => DirectiveSite::Interface(id.walk(schema)),
            DirectiveSiteId::Object(id) => DirectiveSite::Object(id.walk(schema)),
            DirectiveSiteId::Scalar(id) => DirectiveSite::Scalar(id.walk(schema)),
            DirectiveSiteId::Union(id) => DirectiveSite::Union(id.walk(schema)),
        }
    }
}

impl<'a> DirectiveSite<'a> {
    pub fn id(&self) -> DirectiveSiteId {
        match self {
            DirectiveSite::Enum(walker) => DirectiveSiteId::Enum(walker.id),
            DirectiveSite::EnumValue(walker) => DirectiveSiteId::EnumValue(walker.id),
            DirectiveSite::Field(walker) => DirectiveSiteId::Field(walker.id),
            DirectiveSite::InputObject(walker) => DirectiveSiteId::InputObject(walker.id),
            DirectiveSite::InputValue(walker) => DirectiveSiteId::InputValue(walker.id),
            DirectiveSite::Interface(walker) => DirectiveSiteId::Interface(walker.id),
            DirectiveSite::Object(walker) => DirectiveSiteId::Object(walker.id),
            DirectiveSite::Scalar(walker) => DirectiveSiteId::Scalar(walker.id),
            DirectiveSite::Union(walker) => DirectiveSiteId::Union(walker.id),
        }
    }
    pub fn is_enum(&self) -> bool {
        matches!(self, DirectiveSite::Enum(_))
    }
    pub fn as_enum(&self) -> Option<EnumDefinition<'a>> {
        match self {
            DirectiveSite::Enum(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_enum_value(&self) -> bool {
        matches!(self, DirectiveSite::EnumValue(_))
    }
    pub fn as_enum_value(&self) -> Option<EnumValue<'a>> {
        match self {
            DirectiveSite::EnumValue(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_field(&self) -> bool {
        matches!(self, DirectiveSite::Field(_))
    }
    pub fn as_field(&self) -> Option<FieldDefinition<'a>> {
        match self {
            DirectiveSite::Field(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_input_object(&self) -> bool {
        matches!(self, DirectiveSite::InputObject(_))
    }
    pub fn as_input_object(&self) -> Option<InputObjectDefinition<'a>> {
        match self {
            DirectiveSite::InputObject(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_input_value(&self) -> bool {
        matches!(self, DirectiveSite::InputValue(_))
    }
    pub fn as_input_value(&self) -> Option<InputValueDefinition<'a>> {
        match self {
            DirectiveSite::InputValue(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_interface(&self) -> bool {
        matches!(self, DirectiveSite::Interface(_))
    }
    pub fn as_interface(&self) -> Option<InterfaceDefinition<'a>> {
        match self {
            DirectiveSite::Interface(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_object(&self) -> bool {
        matches!(self, DirectiveSite::Object(_))
    }
    pub fn as_object(&self) -> Option<ObjectDefinition<'a>> {
        match self {
            DirectiveSite::Object(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_scalar(&self) -> bool {
        matches!(self, DirectiveSite::Scalar(_))
    }
    pub fn as_scalar(&self) -> Option<ScalarDefinition<'a>> {
        match self {
            DirectiveSite::Scalar(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_union(&self) -> bool {
        matches!(self, DirectiveSite::Union(_))
    }
    pub fn as_union(&self) -> Option<UnionDefinition<'a>> {
        match self {
            DirectiveSite::Union(item) => Some(*item),
            _ => None,
        }
    }
}
