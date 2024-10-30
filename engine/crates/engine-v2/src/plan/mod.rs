mod adapter;
mod build;
mod error;
mod generated;
mod prelude;

use std::num::NonZero;

use crate::operation::{Operation, QueryInputValues, Variables};
use adapter::OperationAdapter;
pub(crate) use generated::*;
use schema::Schema;
use walker::Walk;

pub type PlanResult<T> = Result<T, error::PlanError>;

#[allow(unused)]
pub fn plan(schema: &Schema, mut operation: Operation) -> PlanResult<OperationPlan> {
    let graph = query_planning::OperationGraph::new(schema, OperationAdapter::new(schema, &mut operation))?.solve()?;
    OperationPlan::build(schema, operation, graph)
}

#[derive(Clone, Copy)]
pub(crate) struct PlanContext<'a> {
    pub(super) schema: &'a Schema,
    pub(super) operation_plan: &'a OperationPlan,
    pub(super) variables: &'a Variables,
}

#[derive(id_derives::IndexedFields)]
pub(crate) struct OperationPlan {
    #[indexed_by(FieldId)]
    fields: Vec<FieldRecord>,
    #[indexed_by(FieldArgumentId)]
    field_arguments: Vec<FieldArgumentRecord>,
    #[indexed_by(VariableDefinitionId)]
    variable_definitions: Vec<VariableDefinitionRecord>,
    #[indexed_by(PlanId)]
    plans: Vec<PlanRecord>,
    #[indexed_by(FieldRefId)]
    field_refs: Vec<FieldId>,
    // deduplicated by rule
    pub query_modifiers: Vec<QueryModifierRecord>,
    // deduplicated by rule
    pub response_modifiers: Vec<ResponseModifierRecord>,
    pub query_input_values: QueryInputValues,
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
