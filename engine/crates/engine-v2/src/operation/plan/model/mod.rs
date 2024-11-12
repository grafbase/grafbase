#![allow(unused)]
mod executable;
mod field;
mod generated;
mod plan;
mod prelude;
mod query_partition;
mod selection_set;

use std::sync::Arc;

use schema::{EntityDefinitionId, FieldSetRecord, Schema};

use crate::{
    operation::{OperationSolution, OperationSolutionContext, ResponseModifierRule, Variables},
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
    pub operation_solution: &'a OperationSolution,
    pub operation_plan: &'a OperationPlan,
}

impl<'ctx> From<OperationPlanContext<'ctx>> for OperationSolutionContext<'ctx> {
    fn from(ctx: OperationPlanContext<'ctx>) -> Self {
        OperationSolutionContext {
            schema: ctx.schema,
            operation_solution: ctx.operation_solution,
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
        &ctx.operation_solution.shapes
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
