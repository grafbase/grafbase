//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    StringId,
    generated::{
        FieldDefinition, FieldDefinitionId, InterfaceDefinition, InterfaceDefinitionId, JoinImplementsDefinition,
        JoinImplementsDefinitionRecord, Subgraph, SubgraphId, TypeSystemDirective, TypeSystemDirectiveId,
    },
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ObjectDefinition @meta(module: "object", debug: false) @indexed(deduplicated: true, id_size: "u32") {
///   name: String!
///   description: String
///   interfaces: [InterfaceDefinition!]!
///   directives: [TypeSystemDirective!]!
///   fields: [FieldDefinition!]!
///   "sorted by SubgraphId, then InterfaceId"
///   join_implements: [JoinImplementsDefinition!]!
///   "sorted by SubgraphId"
///   exists_in_subgraphs: [Subgraph!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ObjectDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub interface_ids: Vec<InterfaceDefinitionId>,
    pub directive_ids: Vec<TypeSystemDirectiveId>,
    pub field_ids: IdRange<FieldDefinitionId>,
    /// sorted by SubgraphId, then InterfaceId
    pub join_implement_records: Vec<JoinImplementsDefinitionRecord>,
    /// sorted by SubgraphId
    pub exists_in_subgraph_ids: Vec<SubgraphId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ObjectDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct ObjectDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub id: ObjectDefinitionId,
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
        self.as_ref().field_ids.walk(self.schema)
    }
    /// sorted by SubgraphId, then InterfaceId
    pub fn join_implements(&self) -> impl Iter<Item = JoinImplementsDefinition<'a>> + 'a {
        self.as_ref().join_implement_records.walk(self.schema)
    }
    /// sorted by SubgraphId
    pub fn exists_in_subgraphs(&self) -> impl Iter<Item = Subgraph<'a>> + 'a {
        self.as_ref().exists_in_subgraph_ids.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for ObjectDefinitionId {
    type Walker<'w>
        = ObjectDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ObjectDefinition {
            schema: schema.into(),
            id: self,
        }
    }
}
