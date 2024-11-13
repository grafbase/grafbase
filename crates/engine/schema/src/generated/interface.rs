//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{
        FieldDefinition, FieldDefinitionId, ObjectDefinition, ObjectDefinitionId, Subgraph, SubgraphId,
        TypeSystemDirective, TypeSystemDirectiveId,
    },
    prelude::*,
    StringId,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type InterfaceDefinition
///   @meta(module: "interface", debug: false)
///   @indexed(deduplicated: true, id_size: "u32", max_id: "MAX_ID") {
///   name: String!
///   description: String
///   fields: [FieldDefinition!]!
///   interfaces: [InterfaceDefinition!]!
///   "sorted by ObjectId"
///   possible_types: [ObjectDefinition!]!
///   possible_types_ordered_by_typename: [ObjectDefinition!]!
///   directives: [TypeSystemDirective!]!
///   """
///   It can happen that an object that implements this interface at the supergraph level doesn't in a subgraph.
///   If this happens, we keep track of those subgraph so we we can generate appropriate queries.
///   Sorted by SubgraphId
///   """
///   not_fully_implemented_in: [Subgraph!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InterfaceDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub field_ids: IdRange<FieldDefinitionId>,
    pub interface_ids: Vec<InterfaceDefinitionId>,
    /// sorted by ObjectId
    pub possible_type_ids: Vec<ObjectDefinitionId>,
    pub possible_types_ordered_by_typename_ids: Vec<ObjectDefinitionId>,
    pub directive_ids: Vec<TypeSystemDirectiveId>,
    /// It can happen that an object that implements this interface at the supergraph level doesn't in a subgraph.
    /// If this happens, we keep track of those subgraph so we we can generate appropriate queries.
    /// Sorted by SubgraphId
    pub not_fully_implemented_in_ids: Vec<SubgraphId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct InterfaceDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct InterfaceDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub id: InterfaceDefinitionId,
}

impl std::ops::Deref for InterfaceDefinition<'_> {
    type Target = InterfaceDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> InterfaceDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a InterfaceDefinitionRecord {
        &self.schema[self.id]
    }
    pub fn name(&self) -> &'a str {
        self.name_id.walk(self.schema)
    }
    pub fn description(&self) -> Option<&'a str> {
        self.description_id.walk(self.schema)
    }
    pub fn fields(&self) -> impl Iter<Item = FieldDefinition<'a>> + 'a {
        self.field_ids.walk(self.schema)
    }
    pub fn interfaces(&self) -> impl Iter<Item = InterfaceDefinition<'a>> + 'a {
        self.as_ref().interface_ids.walk(self.schema)
    }
    /// sorted by ObjectId
    pub fn possible_types(&self) -> impl Iter<Item = ObjectDefinition<'a>> + 'a {
        self.as_ref().possible_type_ids.walk(self.schema)
    }
    pub fn possible_types_ordered_by_typename(&self) -> impl Iter<Item = ObjectDefinition<'a>> + 'a {
        self.as_ref().possible_types_ordered_by_typename_ids.walk(self.schema)
    }
    pub fn directives(&self) -> impl Iter<Item = TypeSystemDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.schema)
    }
    /// It can happen that an object that implements this interface at the supergraph level doesn't in a subgraph.
    /// If this happens, we keep track of those subgraph so we we can generate appropriate queries.
    /// Sorted by SubgraphId
    pub fn not_fully_implemented_in(&self) -> impl Iter<Item = Subgraph<'a>> + 'a {
        self.as_ref().not_fully_implemented_in_ids.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for InterfaceDefinitionId {
    type Walker<'w> = InterfaceDefinition<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        InterfaceDefinition {
            schema: schema.into(),
            id: self,
        }
    }
}
