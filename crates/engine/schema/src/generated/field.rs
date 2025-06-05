//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
mod derive;
mod provides;
mod requires;
mod subgraph_type;

use crate::{
    StringId,
    generated::{
        EntityDefinition, EntityDefinitionId, InputValueDefinition, InputValueDefinitionId, ResolverDefinition,
        ResolverDefinitionId, Subgraph, SubgraphId, Type, TypeRecord, TypeSystemDirective, TypeSystemDirectiveId,
    },
    prelude::*,
};
pub use derive::*;
pub use provides::*;
pub use requires::*;
pub use subgraph_type::*;
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type FieldDefinition @meta(module: "field", debug: false) @indexed(id_size: "u32") {
///   name: String!
///   description: String
///   parent_entity: EntityDefinition!
///   ty: Type!
///   resolvers: [ResolverDefinition!]!
///   exists_in_subgraphs: [Subgraph!]!
///   "Present if subgraph has a different type from the supergraph"
///   subgraph_types: [SubgraphType!]!
///   requires: [FieldRequires!]! @field(record_field_name: "requires_records")
///   provides: [FieldProvides!]! @field(record_field_name: "provides_records")
///   "The arguments referenced by this range are sorted by their name (string). Names are NOT unique because of @internal/@require"
///   arguments: [InputValueDefinition!]!
///   directives: [TypeSystemDirective!]!
///   derives: [DeriveDefinition!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FieldDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub parent_entity_id: EntityDefinitionId,
    pub ty_record: TypeRecord,
    pub resolver_ids: Vec<ResolverDefinitionId>,
    pub exists_in_subgraph_ids: Vec<SubgraphId>,
    /// Present if subgraph has a different type from the supergraph
    pub subgraph_type_records: Vec<SubgraphTypeRecord>,
    pub requires_records: Vec<FieldRequiresRecord>,
    pub provides_records: Vec<FieldProvidesRecord>,
    /// The arguments referenced by this range are sorted by their name (string). Names are NOT unique because of @internal/@require
    pub argument_ids: IdRange<InputValueDefinitionId>,
    pub directive_ids: Vec<TypeSystemDirectiveId>,
    pub derive_ids: IdRange<DeriveDefinitionId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct FieldDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct FieldDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub id: FieldDefinitionId,
}

impl std::ops::Deref for FieldDefinition<'_> {
    type Target = FieldDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> FieldDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a FieldDefinitionRecord {
        &self.schema[self.id]
    }
    pub fn name(&self) -> &'a str {
        self.name_id.walk(self.schema)
    }
    pub fn description(&self) -> Option<&'a str> {
        self.description_id.walk(self.schema)
    }
    pub fn parent_entity(&self) -> EntityDefinition<'a> {
        self.parent_entity_id.walk(self.schema)
    }
    pub fn ty(&self) -> Type<'a> {
        self.ty_record.walk(self.schema)
    }
    pub fn resolvers(&self) -> impl Iter<Item = ResolverDefinition<'a>> + 'a {
        self.as_ref().resolver_ids.walk(self.schema)
    }
    pub fn exists_in_subgraphs(&self) -> impl Iter<Item = Subgraph<'a>> + 'a {
        self.as_ref().exists_in_subgraph_ids.walk(self.schema)
    }
    /// Present if subgraph has a different type from the supergraph
    pub fn subgraph_types(&self) -> impl Iter<Item = SubgraphType<'a>> + 'a {
        self.as_ref().subgraph_type_records.walk(self.schema)
    }
    pub fn requires(&self) -> impl Iter<Item = FieldRequires<'a>> + 'a {
        self.as_ref().requires_records.walk(self.schema)
    }
    pub fn provides(&self) -> impl Iter<Item = FieldProvides<'a>> + 'a {
        self.as_ref().provides_records.walk(self.schema)
    }
    /// The arguments referenced by this range are sorted by their name (string). Names are NOT unique because of @internal/@require
    pub fn arguments(&self) -> impl Iter<Item = InputValueDefinition<'a>> + 'a {
        self.as_ref().argument_ids.walk(self.schema)
    }
    pub fn directives(&self) -> impl Iter<Item = TypeSystemDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.schema)
    }
    pub fn derives(&self) -> impl Iter<Item = DeriveDefinition<'a>> + 'a {
        self.as_ref().derive_ids.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for FieldDefinitionId {
    type Walker<'w>
        = FieldDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        FieldDefinition {
            schema: schema.into(),
            id: self,
        }
    }
}
