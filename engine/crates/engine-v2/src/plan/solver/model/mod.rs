mod argument;
mod field;
mod generated;
mod hydrate;
mod prelude;
mod selection_set;

use std::num::NonZero;

use crate::{
    operation::QueryInputValues,
    response::{FieldShapeId, ResponseKeys, Shapes},
};
pub(crate) use generated::*;
pub(crate) use hydrate::*;
use id_newtypes::IdRange;
use schema::{ObjectDefinitionId, Schema};
use walker::{Iter, Walk};

#[derive(Clone, Copy)]
pub(crate) struct OperationSolutionContext<'a> {
    pub schema: &'a Schema,
    pub operation_solution: &'a OperationSolution,
}

impl<'a> From<OperationSolutionContext<'a>> for &'a Schema {
    fn from(ctx: OperationSolutionContext<'a>) -> Self {
        ctx.schema
    }
}

impl<'a> OperationSolutionContext<'a> {
    pub(in crate::plan) fn query_partitions(&self) -> impl Iter<Item = QueryPartition<'a>> + 'a {
        IdRange::<QueryPartitionId>::from(0..self.operation_solution.query_partitions.len()).walk(*self)
    }

    pub(in crate::plan) fn response_modifier_definitions(
        &self,
    ) -> impl Iter<Item = ResponseModifierDefinition<'a>> + 'a {
        IdRange::<ResponseModifierDefinitionId>::from(0..self.operation_solution.response_modifier_definitions.len())
            .walk(*self)
    }
}

#[derive(id_derives::IndexedFields, serde::Serialize, serde::Deserialize)]
pub(crate) struct OperationSolution {
    pub root_object_id: ObjectDefinitionId,
    #[indexed_by(DataFieldId)]
    pub data_fields: Vec<DataFieldRecord>,
    #[indexed_by(TypenameFieldId)]
    pub typename_fields: Vec<TypenameFieldRecord>,
    #[indexed_by(FieldArgumentId)]
    pub field_arguments: Vec<FieldArgumentRecord>,
    #[indexed_by(VariableDefinitionId)]
    pub variable_definitions: Vec<VariableDefinitionRecord>,
    #[indexed_by(QueryPartitionId)]
    pub query_partitions: Vec<QueryPartitionRecord>,
    pub mutation_partition_order: Vec<QueryPartitionId>,
    #[indexed_by(ResponseObjectSetDefinitionId)]
    pub response_object_set_definitions: Vec<ResponseObjectSetDefinitionRecord>,
    pub response_keys: ResponseKeys,
    // deduplicated by rule
    pub query_modifier_definitions: Vec<QueryModifierDefinitionRecord>,
    // deduplicated by rule
    #[indexed_by(ResponseModifierDefinitionId)]
    pub response_modifier_definitions: Vec<ResponseModifierDefinitionRecord>,
    pub query_input_values: QueryInputValues,
    pub shapes: Shapes,

    // Refs are used to replace a Vec<XId> with a IdRange<XRefId>. IdRange<XRefId> will at most have a size
    // of 2 * u32 while Vec<XId> is 3 words long. And we store everything in a single Vec.
    #[indexed_by(FieldRefId)]
    pub field_refs: Vec<FieldId>,
    #[indexed_by(DataFieldRefId)]
    pub data_field_refs: Vec<DataFieldId>,
    #[indexed_by(FieldShapeRefId)]
    pub field_shape_refs: Vec<FieldShapeId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct FieldRefId(NonZero<u32>);

impl<'a> Walk<OperationSolutionContext<'a>> for FieldRefId {
    type Walker<'w> = Field<'w> where 'a: 'w;

    fn walk<'w>(self, ctx: impl Into<OperationSolutionContext<'a>>) -> Field<'w>
    where
        'a: 'w,
    {
        let ctx: OperationSolutionContext<'a> = ctx.into();
        ctx.operation_solution[self].walk(ctx)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct DataFieldRefId(NonZero<u32>);

impl<'a> Walk<OperationSolutionContext<'a>> for DataFieldRefId {
    type Walker<'w> = DataField<'w> where 'a: 'w;

    fn walk<'w>(self, ctx: impl Into<OperationSolutionContext<'a>>) -> DataField<'w>
    where
        'a: 'w,
    {
        let ctx: OperationSolutionContext<'a> = ctx.into();
        ctx.operation_solution[self].walk(ctx)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct FieldShapeRefId(NonZero<u32>);
