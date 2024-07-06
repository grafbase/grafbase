use schema::SchemaWalker;

use crate::{
    execution::{ExecutionPlan, ExecutionPlanId, ExecutionPlans},
    operation::{Operation, OperationWalker, QueryInputValueId, QueryInputValueWalker, Variables},
    response::{ResponseKeys, Shapes},
};

mod field;
mod selection_set;

pub use field::*;
pub use selection_set::*;

/// TODO: Context is really big...
#[derive(Clone, Copy)]
pub(crate) struct PlanWalker<'a, Item = (), SchemaItem = ()> {
    pub(super) schema_walker: SchemaWalker<'a, SchemaItem>,
    pub(super) operation: &'a Operation,
    pub(super) variables: &'a Variables,
    pub(super) plans: &'a ExecutionPlans,
    pub(super) execution_plan_id: ExecutionPlanId,
    pub(super) item: Item,
}

// really weird to index through a walker, need to be reworked
impl<'a, I> std::ops::Index<I> for PlanWalker<'a, (), ()>
where
    Operation: std::ops::Index<I>,
{
    type Output = <Operation as std::ops::Index<I>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        &self.operation[index]
    }
}

impl<'a> std::fmt::Debug for PlanWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanWalker").finish_non_exhaustive()
    }
}

impl<'a, I: Copy, SI> PlanWalker<'a, I, SI>
where
    Operation: std::ops::Index<I>,
{
    pub fn as_ref(&self) -> &'a <Operation as std::ops::Index<I>>::Output {
        &self.operation[self.item]
    }
}

impl<'a> PlanWalker<'a, (), ()> {
    pub fn as_ref(&self) -> &'a ExecutionPlan {
        &self.plans[self.execution_plan_id]
    }

    pub fn schema(&self) -> SchemaWalker<'a, ()> {
        self.schema_walker
    }

    pub fn response_keys(&self) -> &'a ResponseKeys {
        &self.operation.response_keys
    }

    pub fn shapes(&self) -> &'a Shapes {
        &self.plans.shapes
    }

    pub fn selection_set(self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::RootFields(self)
    }
}

impl<'a, I, SI> PlanWalker<'a, I, SI> {
    pub fn walk_with<I2, SI2>(&self, item: I2, schema_item: SI2) -> PlanWalker<'a, I2, SI2> {
        PlanWalker {
            operation: self.operation,
            variables: self.variables,
            plans: self.plans,
            execution_plan_id: self.execution_plan_id,
            schema_walker: self.schema_walker.walk(schema_item),
            item,
        }
    }

    fn bound_walk_with<I2, SI2: Copy>(&self, item: I2, schema_item: SI2) -> OperationWalker<'a, I2, SI2> {
        self.operation
            .walker_with(self.schema_walker.walk(schema_item), self.variables)
            .walk(item)
    }
}

impl<'a> PlanWalker<'a, (), ()> {
    pub fn walk_input_value(&self, input_value_id: QueryInputValueId) -> QueryInputValueWalker<'a> {
        self.bound_walk_with(&self.operation[input_value_id], ())
    }
}
