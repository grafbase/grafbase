use schema::SchemaWalker;

use crate::{
    operation::{
        LogicalPlanId, LogicalPlanResponseBlueprint, OperationWalker, PreparedOperation, QueryInputValueId,
        QueryInputValueWalker, ResponseBlueprint, ResponseModifier,
    },
    response::ResponseKeys,
};

mod field;
mod selection_set;

pub use field::*;
pub use selection_set::*;

use super::{ExecutableOperation, ExecutionPlanId};

/// TODO: Context is really big...
#[derive(Clone, Copy)]
pub(crate) struct PlanWalker<'a, Item = (), SchemaItem = ()> {
    pub(super) schema_walker: SchemaWalker<'a, SchemaItem>,
    pub(super) operation: &'a ExecutableOperation,
    pub(super) plan_id: ExecutionPlanId,
    pub(super) item: Item,
}

// really weird to index through a walker, need to be reworked
impl<'a, I> std::ops::Index<I> for PlanWalker<'a, (), ()>
where
    PreparedOperation: std::ops::Index<I>,
{
    type Output = <PreparedOperation as std::ops::Index<I>>::Output;
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
    PreparedOperation: std::ops::Index<I>,
{
    pub fn as_ref(&self) -> &'a <PreparedOperation as std::ops::Index<I>>::Output {
        &self.operation[self.item]
    }
}

impl<'a> PlanWalker<'a, (), ()> {
    pub fn schema(&self) -> SchemaWalker<'a, ()> {
        self.schema_walker
    }

    pub fn response_keys(&self) -> &'a ResponseKeys {
        &self.operation.response_keys
    }

    pub fn blueprint(&self) -> &'a ResponseBlueprint {
        &self.operation.response_blueprint
    }

    pub fn logical_plan(&self) -> LogicalPlanWalker<'a> {
        self.walk_with(self.operation[self.plan_id].logical_plan_id, ())
    }

    pub fn operation(&self) -> &'a ExecutableOperation {
        self.operation
    }

    pub fn selection_set(self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::RootFields(self)
    }
}

impl<'a, I, SI> PlanWalker<'a, I, SI> {
    pub fn walk_with<I2, SI2>(&self, item: I2, schema_item: SI2) -> PlanWalker<'a, I2, SI2> {
        PlanWalker {
            operation: self.operation,
            plan_id: self.plan_id,
            schema_walker: self.schema_walker.walk(schema_item),
            item,
        }
    }

    fn bound_walk_with<I2, SI2: Copy>(&self, item: I2, schema_item: SI2) -> OperationWalker<'a, I2, SI2> {
        self.operation
            .prepared
            .walker_with(self.schema_walker.walk(schema_item), &self.operation.variables)
            .walk(item)
    }
}

impl<'a> PlanWalker<'a, (), ()> {
    pub fn walk_input_value(&self, input_value_id: QueryInputValueId) -> QueryInputValueWalker<'a> {
        self.bound_walk_with(&self.operation.prepared[input_value_id], ())
    }
}

type LogicalPlanWalker<'a> = PlanWalker<'a, LogicalPlanId, ()>;

impl<'a> LogicalPlanWalker<'a> {
    pub fn response_blueprint(&self) -> &LogicalPlanResponseBlueprint {
        &self.operation.response_blueprint[self.item]
    }

    #[allow(unused)]
    pub fn response_modifiers(&self) -> impl Iterator<Item = &'a ResponseModifier> + 'a {
        self.operation[self.response_blueprint().response_modifiers_ids].iter()
    }
}
