mod field;
mod generated;
mod hydrate;
mod prelude;

use std::num::NonZero;

use crate::{
    operation::QueryInputValues,
    response::{FieldShapeId, ResponseKeys, Shapes},
};
pub(crate) use generated::*;
pub(crate) use hydrate::*;
use schema::Schema;
use walker::Walk;

#[derive(Clone, Copy)]
pub(crate) struct PlanContext<'a> {
    pub(super) schema: &'a Schema,
    pub(super) operation_plan: &'a OperationPlan,
}

#[derive(id_derives::IndexedFields)]
pub(crate) struct OperationPlan {
    #[indexed_by(DataPlanFieldId)]
    pub data_fields: Vec<DataPlanFieldRecord>,
    #[indexed_by(TypenamePlanFieldId)]
    pub typename_fields: Vec<TypenamePlanFieldRecord>,
    #[indexed_by(FieldArgumentId)]
    pub field_arguments: Vec<FieldArgumentRecord>,
    #[indexed_by(VariableDefinitionId)]
    pub variable_definitions: Vec<VariableDefinitionRecord>,
    #[indexed_by(PlanId)]
    pub plans: Vec<PlanRecord>,
    #[indexed_by(ResponseObjectSetDefinitionId)]
    pub response_object_set_definitions: Vec<ResponseObjectSetDefinitionRecord>,
    pub response_keys: ResponseKeys,
    // deduplicated by rule
    pub query_modifier_definitions: Vec<QueryModifierDefinitionRecord>,
    // deduplicated by rule
    pub response_modifier_definitions: Vec<ResponseModifierDefinitionRecord>,
    #[allow(unused)]
    pub query_input_values: QueryInputValues,
    pub shapes: Shapes,

    // Refs are used to replace a Vec<XId> with a IdRange<XRefId>. IdRange<XRefId> will at most have a size
    // of 2 * u32 while Vec<XId> is 3 words long. And we store everything in a single Vec.
    #[indexed_by(PlanFieldRefId)]
    pub field_refs: Vec<PlanFieldId>,
    #[indexed_by(DataPlanFieldRefId)]
    pub data_field_refs: Vec<DataPlanFieldId>,
    #[indexed_by(FieldShapeRefId)]
    pub field_shape_refs: Vec<FieldShapeId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct PlanFieldRefId(NonZero<u32>);

impl<'a> Walk<PlanContext<'a>> for PlanFieldRefId {
    type Walker<'w> = PlanField<'w> where 'a: 'w;

    fn walk<'w>(self, ctx: PlanContext<'a>) -> PlanField<'w>
    where
        'a: 'w,
    {
        ctx.operation_plan[self].walk(ctx)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct DataPlanFieldRefId(NonZero<u32>);

impl<'a> Walk<PlanContext<'a>> for DataPlanFieldRefId {
    type Walker<'w> = DataPlanField<'w> where 'a: 'w;

    fn walk<'w>(self, ctx: PlanContext<'a>) -> DataPlanField<'w>
    where
        'a: 'w,
    {
        ctx.operation_plan[self].walk(ctx)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct FieldShapeRefId(NonZero<u32>);
