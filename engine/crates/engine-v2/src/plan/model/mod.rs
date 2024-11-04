mod generated;
mod prelude;

use std::num::NonZero;

use crate::{
    operation::QueryInputValues,
    response::{FieldShapeId, ResponseKeys, Shapes},
};
pub(crate) use generated::*;
use schema::Schema;
use walker::Walk;

#[derive(Clone, Copy)]
pub(crate) struct PlanContext<'a> {
    pub(super) schema: &'a Schema,
    pub(super) operation_plan: &'a OperationPlan,
}

#[derive(id_derives::IndexedFields)]
pub(crate) struct OperationPlan {
    #[indexed_by(DataFieldId)]
    pub data_fields: Vec<DataFieldRecord>,
    #[indexed_by(TypenameFieldId)]
    pub typename_fields: Vec<TypenameFieldRecord>,
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
    pub query_modifiers: Vec<QueryModifierRecord>,
    // deduplicated by rule
    pub response_modifiers: Vec<ResponseModifierRecord>,
    #[allow(unused)]
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

impl<'a> Walk<PlanContext<'a>> for FieldRefId {
    type Walker<'w> = Field<'w> where 'a: 'w;

    fn walk<'w>(self, ctx: PlanContext<'a>) -> Field<'w>
    where
        'a: 'w,
    {
        ctx.operation_plan[self].walk(ctx)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct DataFieldRefId(NonZero<u32>);

impl<'a> Walk<PlanContext<'a>> for DataFieldRefId {
    type Walker<'w> = DataField<'w> where 'a: 'w;

    fn walk<'w>(self, ctx: PlanContext<'a>) -> DataField<'w>
    where
        'a: 'w,
    {
        ctx.operation_plan[self].walk(ctx)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct FieldShapeRefId(NonZero<u32>);
