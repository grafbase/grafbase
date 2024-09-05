use schema::SchemaWalker;

use crate::operation::{
    LogicalPlanId, LogicalPlanResponseBlueprint, PreparedOperation, QueryInputValueId, QueryInputValueWalker,
    QueryModifications, ResponseBlueprint, Variables,
};

mod field;
mod selection_set;

pub use field::*;
pub use selection_set::*;

use super::PreparedOperationWalker;

/// TODO: Context is really big...
#[derive(Clone, Copy)]
pub(crate) struct PlanWalker<'a, Item = (), SchemaItem = ()> {
    pub schema_walker: SchemaWalker<'a, SchemaItem>,
    pub operation: &'a PreparedOperation,
    pub query_modifications: &'a QueryModifications,
    pub variables: &'a Variables,
    pub logical_plan_id: LogicalPlanId,
    pub item: Item,
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

    pub fn blueprint(&self) -> &'a ResponseBlueprint {
        &self.operation.response_blueprint
    }

    pub fn logical_plan(&self) -> LogicalPlanWalker<'a> {
        self.walk_with(self.logical_plan_id, ())
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
            query_modifications: self.query_modifications,
            logical_plan_id: self.logical_plan_id,
            schema_walker: self.schema_walker.walk(schema_item),
            item,
        }
    }

    fn prepared_walk_with<I2, SI2: Copy>(&self, item: I2, schema_item: SI2) -> PreparedOperationWalker<'a, I2, SI2> {
        PreparedOperationWalker {
            schema_walker: self.schema_walker.walk(schema_item),
            operation: self.operation,
            variables: self.variables,
            item,
        }
    }
}

impl<'a> PlanWalker<'a, (), ()> {
    pub fn walk_input_value(&self, input_value_id: QueryInputValueId) -> QueryInputValueWalker<'a> {
        self.prepared_walk_with(&self.operation.query_input_values[input_value_id], ())
    }
}

type LogicalPlanWalker<'a> = PlanWalker<'a, LogicalPlanId, ()>;

impl<'a> LogicalPlanWalker<'a> {
    pub fn response_blueprint(&self) -> &LogicalPlanResponseBlueprint {
        &self.operation.response_blueprint[self.item]
    }
}
