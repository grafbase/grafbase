//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{InputValueDefinition, InputValueDefinitionId, TypeSystemDirective, TypeSystemDirectiveId},
    prelude::*,
    StringId,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type InputObjectDefinition @meta(module: "input_object") @indexed(id_size: "u32") {
///   name: String!
///   description: String
///   input_fields: [InputValueDefinition!]!
///   directives: [TypeSystemDirective!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InputObjectDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub input_field_ids: IdRange<InputValueDefinitionId>,
    pub directive_ids: Vec<TypeSystemDirectiveId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct InputObjectDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct InputObjectDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub id: InputObjectDefinitionId,
}

impl std::ops::Deref for InputObjectDefinition<'_> {
    type Target = InputObjectDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> InputObjectDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a InputObjectDefinitionRecord {
        &self.schema[self.id]
    }
    pub fn name(&self) -> &'a str {
        self.name_id.walk(self.schema)
    }
    pub fn description(&self) -> Option<&'a str> {
        self.description_id.walk(self.schema)
    }
    pub fn input_fields(&self) -> impl Iter<Item = InputValueDefinition<'a>> + 'a {
        self.as_ref().input_field_ids.walk(self.schema)
    }
    pub fn directives(&self) -> impl Iter<Item = TypeSystemDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for InputObjectDefinitionId {
    type Walker<'w>
        = InputObjectDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        InputObjectDefinition {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for InputObjectDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputObjectDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("input_fields", &self.input_fields())
            .field("directives", &self.directives())
            .finish()
    }
}
