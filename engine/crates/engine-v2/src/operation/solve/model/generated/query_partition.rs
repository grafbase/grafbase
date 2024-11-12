//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_solution.graphql
use crate::operation::solve::model::{
    generated::{
        DataField, ResponseObjectSetDefinition, ResponseObjectSetDefinitionId, SelectionSet, SelectionSetRecord,
    },
    prelude::*,
    DataFieldRefId,
};
use schema::{EntityDefinition, EntityDefinitionId, ResolverDefinition, ResolverDefinitionId};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type QueryPartition @indexed(id_size: "u16") @meta(module: "query_partition") {
///   entity_definition: EntityDefinition!
///   resolver_definition: ResolverDefinition!
///   selection_set: SelectionSet!
///   required_scalar_fields: [DataFieldRef!]!
///   input: ResponseObjectSetDefinition!
///   shape_id: ConcreteObjectShapeId!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct QueryPartitionRecord {
    pub entity_definition_id: EntityDefinitionId,
    pub resolver_definition_id: ResolverDefinitionId,
    pub selection_set_record: SelectionSetRecord,
    pub required_scalar_field_ids: IdRange<DataFieldRefId>,
    pub input_id: ResponseObjectSetDefinitionId,
    pub shape_id: ConcreteObjectShapeId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct QueryPartitionId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub(crate) struct QueryPartition<'a> {
    pub(in crate::operation::solve::model) ctx: OperationSolutionContext<'a>,
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
        &self.ctx.operation_solution[self.id]
    }
    pub(crate) fn entity_definition(&self) -> EntityDefinition<'a> {
        self.entity_definition_id.walk(self.ctx)
    }
    pub(crate) fn resolver_definition(&self) -> ResolverDefinition<'a> {
        self.resolver_definition_id.walk(self.ctx)
    }
    pub(crate) fn selection_set(&self) -> SelectionSet<'a> {
        self.selection_set_record.walk(self.ctx)
    }
    pub(crate) fn required_scalar_fields(&self) -> impl Iter<Item = DataField<'a>> + 'a {
        self.required_scalar_field_ids.walk(self.ctx)
    }
    pub(crate) fn input(&self) -> ResponseObjectSetDefinition<'a> {
        self.input_id.walk(self.ctx)
    }
}

impl<'a> Walk<OperationSolutionContext<'a>> for QueryPartitionId {
    type Walker<'w> = QueryPartition<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: impl Into<OperationSolutionContext<'a>>) -> Self::Walker<'w>
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
            .field("required_scalar_fields", &self.required_scalar_fields())
            .field("input", &self.input())
            .field("shape_id", &self.shape_id)
            .finish()
    }
}
