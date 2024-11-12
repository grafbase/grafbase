//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{FieldDefinition, FieldDefinitionId, SchemaFieldArgument, SchemaFieldArgumentId},
    prelude::*,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type SchemaField
///   @meta(module: "field_set/field", derive: ["PartialEq", "Eq", "PartialOrd", "Ord"], debug: false)
///   @indexed(id_size: "u32", deduplicated: true) {
///   definition: FieldDefinition!
///   "Sorted by input value definition id"
///   sorted_arguments: [SchemaFieldArgument!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchemaFieldRecord {
    pub definition_id: FieldDefinitionId,
    /// Sorted by input value definition id
    pub sorted_argument_ids: IdRange<SchemaFieldArgumentId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct SchemaFieldId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct SchemaField<'a> {
    pub(crate) schema: &'a Schema,
    pub id: SchemaFieldId,
}

impl std::ops::Deref for SchemaField<'_> {
    type Target = SchemaFieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> SchemaField<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a SchemaFieldRecord {
        &self.schema[self.id]
    }
    pub fn definition(&self) -> FieldDefinition<'a> {
        self.definition_id.walk(self.schema)
    }
    /// Sorted by input value definition id
    pub fn sorted_arguments(&self) -> impl Iter<Item = SchemaFieldArgument<'a>> + 'a {
        self.sorted_argument_ids.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for SchemaFieldId {
    type Walker<'w> = SchemaField<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        SchemaField {
            schema: schema.into(),
            id: self,
        }
    }
}
