//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{
        FieldDefinition, FieldDefinitionId, InterfaceDefinition, InterfaceDefinitionId, JoinImplementsDefinition,
        JoinImplementsDefinitionRecord, TypeSystemDirective, TypeSystemDirectiveId,
    },
    prelude::*,
    StringId,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ObjectDefinition
///   @meta(module: "object", debug: false)
///   @indexed(deduplicated: true, id_size: "u32", max_id: "MAX_ID") {
///   name: String!
///   description: String
///   interfaces: [InterfaceDefinition!]!
///   directives: [TypeSystemDirective!]!
///   fields: [FieldDefinition!]!
///   "sorted by SubgraphId, then InterfaceId"
///   join_implements: [JoinImplementsDefinition!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ObjectDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub interface_ids: Vec<InterfaceDefinitionId>,
    pub directive_ids: Vec<TypeSystemDirectiveId>,
    pub field_ids: IdRange<FieldDefinitionId>,
    /// sorted by SubgraphId, then InterfaceId
    pub join_implement_records: Vec<JoinImplementsDefinitionRecord>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct ObjectDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct ObjectDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) id: ObjectDefinitionId,
}

impl std::ops::Deref for ObjectDefinition<'_> {
    type Target = ObjectDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> ObjectDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a ObjectDefinitionRecord {
        &self.schema[self.id]
    }
    pub fn id(&self) -> ObjectDefinitionId {
        self.id
    }
    pub fn name(&self) -> &'a str {
        self.name_id.walk(self.schema)
    }
    pub fn description(&self) -> Option<&'a str> {
        self.description_id.walk(self.schema)
    }
    pub fn interfaces(&self) -> impl Iter<Item = InterfaceDefinition<'a>> + 'a {
        self.as_ref().interface_ids.walk(self.schema)
    }
    pub fn directives(&self) -> impl Iter<Item = TypeSystemDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.schema)
    }
    pub fn fields(&self) -> impl Iter<Item = FieldDefinition<'a>> + 'a {
        self.field_ids.walk(self.schema)
    }
    /// sorted by SubgraphId, then InterfaceId
    pub fn join_implements(&self) -> impl Iter<Item = JoinImplementsDefinition<'a>> + 'a {
        self.as_ref().join_implement_records.walk(self.schema)
    }
}

impl Walk<Schema> for ObjectDefinitionId {
    type Walker<'a> = ObjectDefinition<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        ObjectDefinition { schema, id: self }
    }
}
