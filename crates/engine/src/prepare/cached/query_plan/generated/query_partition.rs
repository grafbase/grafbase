//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/query_plan.graphql
use crate::prepare::cached::query_plan::{
    generated::{
        PartitionSelectionSet, PartitionSelectionSetRecord, ResponseObjectSetDefinition, ResponseObjectSetDefinitionId,
    },
    prelude::*,
    RequiredFieldSet, RequiredFieldSetRecord,
};
use schema::{EntityDefinition, EntityDefinitionId, ResolverDefinition, ResolverDefinitionId};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type QueryPartition @indexed(id_size: "u16") @meta(module: "query_partition") {
///   entity_definition: EntityDefinition!
///   resolver_definition: ResolverDefinition!
///   selection_set: PartitionSelectionSet!
///   required_fields: RequiredFieldSet!
///   input: ResponseObjectSetDefinition!
///   shape_id: ConcreteShapeId!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct QueryPartitionRecord {
    pub entity_definition_id: EntityDefinitionId,
    pub resolver_definition_id: ResolverDefinitionId,
    pub selection_set_record: PartitionSelectionSetRecord,
    pub required_fields_record: RequiredFieldSetRecord,
    pub input_id: ResponseObjectSetDefinitionId,
    pub shape_id: ConcreteShapeId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct QueryPartitionId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub(crate) struct QueryPartition<'a> {
    pub(in crate::prepare::cached::query_plan) ctx: CachedOperationContext<'a>,
    pub(crate) id: QueryPartitionId,
}

impl std::ops::Deref for QueryPartition<'_> {
    type Target = QueryPartitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> QueryPartition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a QueryPartitionRecord {
        &self.ctx.cached.query_plan[self.id]
    }
    pub(crate) fn entity_definition(&self) -> EntityDefinition<'a> {
        self.entity_definition_id.walk(self.ctx)
    }
    pub(crate) fn resolver_definition(&self) -> ResolverDefinition<'a> {
        self.resolver_definition_id.walk(self.ctx)
    }
    pub(crate) fn selection_set(&self) -> PartitionSelectionSet<'a> {
        self.selection_set_record.walk(self.ctx)
    }
    pub(crate) fn required_fields(&self) -> RequiredFieldSet<'a> {
        self.as_ref().required_fields_record.walk(self.ctx)
    }
    pub(crate) fn input(&self) -> ResponseObjectSetDefinition<'a> {
        self.input_id.walk(self.ctx)
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for QueryPartitionId {
    type Walker<'w>
        = QueryPartition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        QueryPartition {
            ctx: ctx.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for QueryPartition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryPartition")
            .field("entity_definition", &self.entity_definition())
            .field("resolver_definition", &self.resolver_definition())
            .field("selection_set", &self.selection_set())
            .field("required_fields", &self.required_fields())
            .field("input", &self.input())
            .field("shape_id", &self.shape_id)
            .finish()
    }
}
