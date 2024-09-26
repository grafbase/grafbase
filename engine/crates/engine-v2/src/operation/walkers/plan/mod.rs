use crate::operation::{
    LogicalPlanId, LogicalPlanResponseBlueprint, PreparedOperation, QueryInputValueId, QueryInputValueWalker,
    QueryModifications, ResponseBlueprint, Variables,
};

mod field;
mod selection_set;

pub use field::*;
use schema::Schema;
pub use selection_set::*;

use super::PreparedOperationWalker;

/// TODO: Context is really big...
#[derive(Clone, Copy)]
pub(crate) struct PlanWalker<'a, Item = ()> {
    pub schema: &'a Schema,
    pub operation: &'a PreparedOperation,
    pub query_modifications: &'a QueryModifications,
    pub variables: &'a Variables,
    pub logical_plan_id: LogicalPlanId,
    pub item: Item,
}

// really weird to index through a walker, need to be reworked
impl<'a, I> std::ops::Index<I> for PlanWalker<'a, ()>
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

impl<'a, I: Copy> PlanWalker<'a, I>
where
    PreparedOperation: std::ops::Index<I>,
{
    pub fn as_ref(&self) -> &'a <PreparedOperation as std::ops::Index<I>>::Output {
        &self.operation[self.item]
    }
}

impl<'a> PlanWalker<'a, ()> {
    pub fn schema(&self) -> &'a Schema {
        self.schema
    }

    pub fn blueprint(&self) -> &'a ResponseBlueprint {
        &self.operation.response_blueprint
    }

    pub fn logical_plan(&self) -> LogicalPlanWalker<'a> {
        self.walk(self.logical_plan_id)
    }

    pub fn selection_set(self) -> PlanSelectionSet<'a> {
        PlanSelectionSet::RootFields(self)
    }
}

impl<'a, I> PlanWalker<'a, I> {
    pub fn walk<I2>(&self, item: I2) -> PlanWalker<'a, I2> {
        PlanWalker {
            schema: self.schema,
            operation: self.operation,
            variables: self.variables,
            query_modifications: self.query_modifications,
            logical_plan_id: self.logical_plan_id,
            item,
        }
    }

    fn prepared_walk_with<I2>(&self, item: I2) -> PreparedOperationWalker<'a, I2> {
        PreparedOperationWalker {
            schema: self.schema,
            operation: self.operation,
            variables: self.variables,
            item,
        }
    }
}

impl<'a> PlanWalker<'a, ()> {
    pub fn walk_input_value(&self, input_value_id: QueryInputValueId) -> QueryInputValueWalker<'a> {
        self.prepared_walk_with(&self.operation.query_input_values[input_value_id])
    }
}

type LogicalPlanWalker<'a> = PlanWalker<'a, LogicalPlanId>;

impl<'a> LogicalPlanWalker<'a> {
    pub fn response_blueprint(&self) -> &LogicalPlanResponseBlueprint {
        &self.operation.response_blueprint[self.item]
    }
}
