//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{
        JoinMemberDefinition, JoinMemberDefinitionRecord, ObjectDefinition, ObjectDefinitionId, TypeSystemDirective,
        TypeSystemDirectiveId,
    },
    prelude::*,
    StringId,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type UnionDefinition @meta(module: "union", debug: false) @indexed(id_size: "u32", max_id: "MAX_ID") {
///   name: String!
///   description: String
///   possible_types: [ObjectDefinition!]!
///   possible_types_ordered_by_typename: [ObjectDefinition!]!
///   directives: [TypeSystemDirective!]!
///   "sorted by SubgraphId, then InterfaceId"
///   join_members: [JoinMemberDefinition!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UnionDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub possible_type_ids: Vec<ObjectDefinitionId>,
    pub possible_types_ordered_by_typename_ids: Vec<ObjectDefinitionId>,
    pub directive_ids: Vec<TypeSystemDirectiveId>,
    /// sorted by SubgraphId, then InterfaceId
    pub join_member_records: Vec<JoinMemberDefinitionRecord>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct UnionDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct UnionDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) id: UnionDefinitionId,
}

impl std::ops::Deref for UnionDefinition<'_> {
    type Target = UnionDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> UnionDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a UnionDefinitionRecord {
        &self.schema[self.id]
    }
    pub fn id(&self) -> UnionDefinitionId {
        self.id
    }
    pub fn name(&self) -> &'a str {
        self.name_id.walk(self.schema)
    }
    pub fn description(&self) -> Option<&'a str> {
        self.description_id.walk(self.schema)
    }
    pub fn possible_types(&self) -> impl Iter<Item = ObjectDefinition<'a>> + 'a {
        self.as_ref().possible_type_ids.walk(self.schema)
    }
    pub fn possible_types_ordered_by_typename(&self) -> impl Iter<Item = ObjectDefinition<'a>> + 'a {
        self.as_ref().possible_types_ordered_by_typename_ids.walk(self.schema)
    }
    pub fn directives(&self) -> impl Iter<Item = TypeSystemDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.schema)
    }
    /// sorted by SubgraphId, then InterfaceId
    pub fn join_members(&self) -> impl Iter<Item = JoinMemberDefinition<'a>> + 'a {
        self.as_ref().join_member_records.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for UnionDefinitionId {
    type Walker<'w> = UnionDefinition<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: &'a Schema) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        UnionDefinition { schema, id: self }
    }
}
