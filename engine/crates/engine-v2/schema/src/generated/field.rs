//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
mod provides;
mod requires;

use crate::{
    generated::{
        EntityDefinition, EntityDefinitionId, InputValueDefinition, InputValueDefinitionId, ResolverDefinition,
        ResolverDefinitionId, Subgraph, SubgraphId, Type, TypeRecord, TypeSystemDirective, TypeSystemDirectiveId,
    },
    prelude::*,
    StringId,
};
pub use provides::*;
pub use requires::*;
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type FieldDefinition @meta(module: "field") @indexed(id_size: "u32", max_id: "MAX_ID") {
///   name: String!
///   description: String
///   parent_entity: EntityDefinition! @meta(debug: false)
///   ty: Type!
///   resolvers: [ResolverDefinition!]!
///   """
///   By default a field is considered shared and providable by *any* subgraph that exposes it.
///   It's up to the composition to ensure it. If this field is specific to some subgraphs, they
///   will be specified in this Vec.
///   """
///   only_resolvable_in: [Subgraph!]!
///   requires: [FieldRequires!]! @field(record_field_name: "requires_records")
///   provides: [FieldProvides!]! @field(record_field_name: "provides_records")
///   "The arguments referenced by this range are sorted by their name (string)"
///   arguments: [InputValueDefinition!]!
///   directives: [TypeSystemDirective!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FieldDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub parent_entity_id: EntityDefinitionId,
    pub ty_record: TypeRecord,
    pub resolver_ids: Vec<ResolverDefinitionId>,
    /// By default a field is considered shared and providable by *any* subgraph that exposes it.
    /// It's up to the composition to ensure it. If this field is specific to some subgraphs, they
    /// will be specified in this Vec.
    pub only_resolvable_in_ids: Vec<SubgraphId>,
    pub requires_records: Vec<FieldRequiresRecord>,
    pub provides_records: Vec<FieldProvidesRecord>,
    /// The arguments referenced by this range are sorted by their name (string)
    pub argument_ids: IdRange<InputValueDefinitionId>,
    pub directive_ids: Vec<TypeSystemDirectiveId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct FieldDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct FieldDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) id: FieldDefinitionId,
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
    pub fn id(&self) -> FieldDefinitionId {
        self.id
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
    /// By default a field is considered shared and providable by *any* subgraph that exposes it.
    /// It's up to the composition to ensure it. If this field is specific to some subgraphs, they
    /// will be specified in this Vec.
    pub fn only_resolvable_in(&self) -> impl Iter<Item = Subgraph<'a>> + 'a {
        self.as_ref().only_resolvable_in_ids.walk(self.schema)
    }
    pub fn requires(&self) -> impl Iter<Item = FieldRequires<'a>> + 'a {
        self.as_ref().requires_records.walk(self.schema)
    }
    pub fn provides(&self) -> impl Iter<Item = FieldProvides<'a>> + 'a {
        self.as_ref().provides_records.walk(self.schema)
    }
    /// The arguments referenced by this range are sorted by their name (string)
    pub fn arguments(&self) -> impl Iter<Item = InputValueDefinition<'a>> + 'a {
        self.argument_ids.walk(self.schema)
    }
    pub fn directives(&self) -> impl Iter<Item = TypeSystemDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.schema)
    }
}

impl Walk<Schema> for FieldDefinitionId {
    type Walker<'a> = FieldDefinition<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        FieldDefinition { schema, id: self }
    }
}

impl std::fmt::Debug for FieldDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("ty", &self.ty().to_string())
            .field(
                "resolvers",
                &self.resolvers().map(|walker| walker.to_string()).collect::<Vec<_>>(),
            )
            .field(
                "only_resolvable_in",
                &self
                    .only_resolvable_in()
                    .map(|walker| walker.to_string())
                    .collect::<Vec<_>>(),
            )
            .field(
                "requires",
                &self.requires().map(|walker| walker.to_string()).collect::<Vec<_>>(),
            )
            .field(
                "provides",
                &self.provides().map(|walker| walker.to_string()).collect::<Vec<_>>(),
            )
            .field(
                "arguments",
                &self.arguments().map(|walker| walker.to_string()).collect::<Vec<_>>(),
            )
            .field(
                "directives",
                &self.directives().map(|walker| walker.to_string()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
