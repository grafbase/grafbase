mod argument;
mod field;
mod generated;
mod hydrate;
mod prelude;
mod selection_set;

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
pub(crate) struct SolvedOperationContext<'a> {
    pub schema: &'a Schema,
    pub operation: &'a SolvedOperation,
}

impl<'a> From<SolvedOperationContext<'a>> for &'a Schema {
    fn from(ctx: SolvedOperationContext<'a>) -> Self {
        ctx.schema
    }
}

impl<'a> SolvedOperationContext<'a> {
    pub(in crate::operation) fn query_partitions(&self) -> impl Iter<Item = QueryPartition<'a>> + 'a {
        IdRange::<QueryPartitionId>::from(0..self.operation.query_partitions.len()).walk(*self)
    }

    pub(in crate::operation) fn response_modifier_definitions(
        &self,
    ) -> impl Iter<Item = ResponseModifierDefinition<'a>> + 'a {
        IdRange::<ResponseModifierDefinitionId>::from(0..self.operation.response_modifier_definitions.len()).walk(*self)
    }
}

/// The solved operation contains almost all the necessary data to execute the operation. It only
/// needs to be adjusted with `@skip`, `@include` etc.. This latter step produces the
/// OperationPlan. If the operation doesn't involve any skip, include or authorized directive it's
/// effectively all the information there is to know.
///
/// The solved operation is fundamentally a partitioning of the original query into QueryPartitions,
/// each associated with a ResolverDefinition and field/object shapes. The latter define the
/// structure we expect to retrieve from the subgraph response.
///
/// Only query partitions resolving root fields in a mutation are ordered. Otherwise there is no
/// direct relationship between them. Dependencies are tracked at the field level:
/// - ResolverDefinition requirements -> `QueryPartition.required_fields`
/// - `@requires` -> `DataField.required_fields`
/// - `@authorized` requirements -> `DataField.required_fields_by_supergraph`
///
/// When building the OperationPlan, taking into account skip, include and unauthorized fields, we
/// infer the ordering of the plans and response modifiers from those requirements. This allows us
/// to run as efficiently as possible the different steps of the plan, only waiting for relevant
/// data.
#[derive(id_derives::IndexedFields, serde::Serialize, serde::Deserialize)]
pub(crate) struct SolvedOperation {
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
pub struct FieldRefId(u32);

impl<'a> Walk<SolvedOperationContext<'a>> for FieldRefId {
    type Walker<'w> = Field<'w> where 'a: 'w;

    fn walk<'w>(self, ctx: impl Into<SolvedOperationContext<'a>>) -> Field<'w>
    where
        'a: 'w,
    {
        let ctx: SolvedOperationContext<'a> = ctx.into();
        ctx.operation[self].walk(ctx)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct DataFieldRefId(u32);

impl<'a> Walk<SolvedOperationContext<'a>> for DataFieldRefId {
    type Walker<'w> = DataField<'w> where 'a: 'w;

    fn walk<'w>(self, ctx: impl Into<SolvedOperationContext<'a>>) -> DataField<'w>
    where
        'a: 'w,
    {
        let ctx: SolvedOperationContext<'a> = ctx.into();
        ctx.operation[self].walk(ctx)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct FieldShapeRefId(u32);
