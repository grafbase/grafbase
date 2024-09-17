//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{FieldDefinition, FieldDefinitionId, InputValueDefinition, InputValueDefinitionId},
    prelude::*,
    SchemaInputValue, SchemaInputValueId,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type RequiredField
///   @meta(module: "field_set", derive: ["PartialEq", "Eq", "PartialOrd", "Ord"], debug: false)
///   @indexed(id_size: "u32", deduplicated: true) {
///   definition: FieldDefinition!
///   arguments: [RequiredFieldArgument!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequiredFieldRecord {
    pub definition_id: FieldDefinitionId,
    pub argument_records: Vec<RequiredFieldArgumentRecord>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct RequiredFieldId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct RequiredField<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) id: RequiredFieldId,
}

impl std::ops::Deref for RequiredField<'_> {
    type Target = RequiredFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> RequiredField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a RequiredFieldRecord {
        &self.schema[self.id]
    }
    pub fn id(&self) -> RequiredFieldId {
        self.id
    }
    pub fn definition(&self) -> FieldDefinition<'a> {
        self.definition_id.walk(self.schema)
    }
    pub fn arguments(&self) -> impl Iter<Item = RequiredFieldArgument<'a>> + 'a {
        self.as_ref().argument_records.walk(self.schema)
    }
}

impl Walk<Schema> for RequiredFieldId {
    type Walker<'a> = RequiredField<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        RequiredField { schema, id: self }
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type RequiredFieldArgument @meta(module: "field_set", derive: ["PartialEq", "Eq", "PartialOrd", "Ord"]) @copy {
///   definition: InputValueDefinition!
///   value: SchemaInputValue!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct RequiredFieldArgumentRecord {
    pub definition_id: InputValueDefinitionId,
    pub value_id: SchemaInputValueId,
}

#[derive(Clone, Copy)]
pub struct RequiredFieldArgument<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: RequiredFieldArgumentRecord,
}

impl std::ops::Deref for RequiredFieldArgument<'_> {
    type Target = RequiredFieldArgumentRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> RequiredFieldArgument<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &RequiredFieldArgumentRecord {
        &self.item
    }
    pub fn definition(&self) -> InputValueDefinition<'a> {
        self.definition_id.walk(self.schema)
    }
    pub fn value(&self) -> SchemaInputValue<'a> {
        self.value_id.walk(self.schema)
    }
}

impl Walk<Schema> for RequiredFieldArgumentRecord {
    type Walker<'a> = RequiredFieldArgument<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        RequiredFieldArgument { schema, item: self }
    }
}

impl std::fmt::Debug for RequiredFieldArgument<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequiredFieldArgument")
            .field("definition", &self.definition().to_string())
            .field("value", &self.value())
            .finish()
    }
}
