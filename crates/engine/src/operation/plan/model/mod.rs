#![allow(unused)]
mod executable;
mod field;
mod generated;
mod plan;
mod prelude;
mod query_partition;
mod selection_set;

use std::sync::Arc;

use id_newtypes::IdRange;
use schema::{EntityDefinitionId, FieldSetRecord, Schema};
use walker::{Iter, Walk};

use crate::{
    operation::{ResponseModifierRule, SolvedOperation, SolvedOperationContext, Variables},
    resolver::Resolver,
    response::{ResponseKey, Shapes},
};

use super::QueryModifications;

pub(crate) use field::*;
pub(crate) use generated::*;
pub(crate) use plan::*;
pub(crate) use query_partition::*;
pub(crate) use selection_set::*;

#[derive(Clone, Copy)]
pub(crate) struct OperationPlanContext<'a> {
    pub schema: &'a Schema,
    pub solved_operation: &'a SolvedOperation,
    pub operation_plan: &'a OperationPlan,
}

impl<'ctx> From<OperationPlanContext<'ctx>> for SolvedOperationContext<'ctx> {
    fn from(ctx: OperationPlanContext<'ctx>) -> Self {
        SolvedOperationContext {
            schema: ctx.schema,
            operation: ctx.solved_operation,
        }
    }
}

impl<'ctx> From<OperationPlanContext<'ctx>> for &'ctx Schema {
    fn from(ctx: OperationPlanContext<'ctx>) -> Self {
        ctx.schema
    }
}

impl<'ctx> From<OperationPlanContext<'ctx>> for &'ctx Shapes {
    fn from(ctx: OperationPlanContext<'ctx>) -> Self {
        &ctx.solved_operation.shapes
    }
}

impl<'a> OperationPlanContext<'a> {
    pub fn plans(&self) -> impl Iter<Item = Plan<'a>> + 'a {
        IdRange::<PlanId>::from(0..self.operation_plan.plans.len()).walk(*self)
    }
}

#[derive(id_derives::IndexedFields)]
pub(crate) struct OperationPlan {
    pub query_modifications: QueryModifications,
    #[indexed_by(PlanId)]
    pub plans: Vec<PlanRecord>,
    #[indexed_by(ResponseModifierId)]
    pub response_modifiers: Vec<ResponseModifierRecord>,
}
