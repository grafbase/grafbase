//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{InputValueDefinition, InputValueDefinitionId},
    prelude::*,
    SchemaInputValue, SchemaInputValueId,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type SchemaFieldArgument
///   @meta(module: "field_set/argument", derive: ["PartialEq", "Eq", "PartialOrd", "Ord"])
///   @indexed(id_size: "u32") {
///   definition: InputValueDefinition!
///   value: SchemaInputValue!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaFieldArgumentRecord {
    pub definition_id: InputValueDefinitionId,
    pub value_id: SchemaInputValueId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct SchemaFieldArgumentId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct SchemaFieldArgument<'a> {
    pub(crate) schema: &'a Schema,
    pub id: SchemaFieldArgumentId,
}

impl std::ops::Deref for SchemaFieldArgument<'_> {
    type Target = SchemaFieldArgumentRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> SchemaFieldArgument<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a SchemaFieldArgumentRecord {
        &self.schema[self.id]
    }
    pub fn definition(&self) -> InputValueDefinition<'a> {
        self.definition_id.walk(self.schema)
    }
    pub fn value(&self) -> SchemaInputValue<'a> {
        self.value_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for SchemaFieldArgumentId {
    type Walker<'w>
        = SchemaFieldArgument<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        SchemaFieldArgument {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for SchemaFieldArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SchemaFieldArgument")
            .field("definition", &self.definition())
            .field("value", &self.value())
            .finish()
    }
}
