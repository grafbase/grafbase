//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{FieldDefinition, FieldDefinitionId, InputObjectDefinition, InputObjectDefinitionId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union InputValueParentDefinition @id @meta(module: "input_value/parent") @variants(remove_suffix: "Definition") =
///   | FieldDefinition
///   | InputObjectDefinition
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputValueParentDefinitionId {
    Field(FieldDefinitionId),
    InputObject(InputObjectDefinitionId),
}

impl std::fmt::Debug for InputValueParentDefinitionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputValueParentDefinitionId::Field(variant) => variant.fmt(f),
            InputValueParentDefinitionId::InputObject(variant) => variant.fmt(f),
        }
    }
}

impl From<FieldDefinitionId> for InputValueParentDefinitionId {
    fn from(value: FieldDefinitionId) -> Self {
        InputValueParentDefinitionId::Field(value)
    }
}
impl From<InputObjectDefinitionId> for InputValueParentDefinitionId {
    fn from(value: InputObjectDefinitionId) -> Self {
        InputValueParentDefinitionId::InputObject(value)
    }
}

impl InputValueParentDefinitionId {
    pub fn is_field(&self) -> bool {
        matches!(self, InputValueParentDefinitionId::Field(_))
    }
    pub fn as_field(&self) -> Option<FieldDefinitionId> {
        match self {
            InputValueParentDefinitionId::Field(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_input_object(&self) -> bool {
        matches!(self, InputValueParentDefinitionId::InputObject(_))
    }
    pub fn as_input_object(&self) -> Option<InputObjectDefinitionId> {
        match self {
            InputValueParentDefinitionId::InputObject(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum InputValueParentDefinition<'a> {
    Field(FieldDefinition<'a>),
    InputObject(InputObjectDefinition<'a>),
}

impl std::fmt::Debug for InputValueParentDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputValueParentDefinition::Field(variant) => variant.fmt(f),
            InputValueParentDefinition::InputObject(variant) => variant.fmt(f),
        }
    }
}

impl<'a> From<FieldDefinition<'a>> for InputValueParentDefinition<'a> {
    fn from(item: FieldDefinition<'a>) -> Self {
        InputValueParentDefinition::Field(item)
    }
}
impl<'a> From<InputObjectDefinition<'a>> for InputValueParentDefinition<'a> {
    fn from(item: InputObjectDefinition<'a>) -> Self {
        InputValueParentDefinition::InputObject(item)
    }
}

impl<'a> Walk<&'a Schema> for InputValueParentDefinitionId {
    type Walker<'w>
        = InputValueParentDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let schema: &'a Schema = schema.into();
        match self {
            InputValueParentDefinitionId::Field(id) => InputValueParentDefinition::Field(id.walk(schema)),
            InputValueParentDefinitionId::InputObject(id) => InputValueParentDefinition::InputObject(id.walk(schema)),
        }
    }
}

impl<'a> InputValueParentDefinition<'a> {
    pub fn id(&self) -> InputValueParentDefinitionId {
        match self {
            InputValueParentDefinition::Field(walker) => InputValueParentDefinitionId::Field(walker.id),
            InputValueParentDefinition::InputObject(walker) => InputValueParentDefinitionId::InputObject(walker.id),
        }
    }
    pub fn is_field(&self) -> bool {
        matches!(self, InputValueParentDefinition::Field(_))
    }
    pub fn as_field(&self) -> Option<FieldDefinition<'a>> {
        match self {
            InputValueParentDefinition::Field(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_input_object(&self) -> bool {
        matches!(self, InputValueParentDefinition::InputObject(_))
    }
    pub fn as_input_object(&self) -> Option<InputObjectDefinition<'a>> {
        match self {
            InputValueParentDefinition::InputObject(item) => Some(*item),
            _ => None,
        }
    }
}
