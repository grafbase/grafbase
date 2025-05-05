mod executable;
mod field;
mod generated;
mod plan;
mod prelude;
mod query_partition;
mod selection_set;

use id_newtypes::IdRange;
use schema::Schema;
use walker::{Iter, Walk};

use crate::prepare::{CachedOperation, CachedOperationContext, PreparedOperation, Shapes};

use super::QueryModifications;

pub(crate) use field::*;
pub(crate) use generated::*;
pub(crate) use query_partition::*;
pub(crate) use selection_set::*;

#[derive(Clone, Copy)]
pub(crate) struct OperationPlanContext<'a> {
    pub schema: &'a Schema,
    pub cached: &'a CachedOperation,
    pub plan: &'a OperationPlan,
}

impl<'ctx> From<(&'ctx Schema, &'ctx PreparedOperation)> for OperationPlanContext<'ctx> {
    fn from((schema, operation): (&'ctx Schema, &'ctx PreparedOperation)) -> Self {
        OperationPlanContext {
            schema,
            cached: &operation.cached,
            plan: &operation.plan,
        }
    }
}

impl<'ctx> From<OperationPlanContext<'ctx>> for CachedOperationContext<'ctx> {
    fn from(ctx: OperationPlanContext<'ctx>) -> Self {
        CachedOperationContext {
            schema: ctx.schema,
            cached: ctx.cached,
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
        &ctx.cached.shapes
    }
}

impl<'a> OperationPlanContext<'a> {
    pub fn plans(&self) -> impl Iter<Item = Plan<'a>> + 'a {
        IdRange::<PlanId>::from(0..self.plan.plans.len()).walk(*self)
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
