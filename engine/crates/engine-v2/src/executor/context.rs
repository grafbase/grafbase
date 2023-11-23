use schema::Names;

use crate::{
    plan::{OperationPlan, PlanId, PlannedSelectionSetWalker},
    Engine,
};

/// Data available during the executor life during its build & execution phases.
#[derive(Clone)]
pub struct ExecutorContext<'a> {
    pub engine: &'a Engine,
    pub plan: &'a OperationPlan,
    pub plan_id: PlanId,
}

impl<'ctx> ExecutorContext<'ctx> {
    /// If you do no need to rename anything, use this walker with the schema names.
    pub fn default_walk_selection_set(&self) -> PlannedSelectionSetWalker<'_> {
        self.walk_selection_set(self.engine.schema.as_ref())
    }

    pub fn walk_selection_set<'a>(&self, names: &'a dyn Names) -> PlannedSelectionSetWalker<'a>
    where
        'ctx: 'a,
    {
        PlannedSelectionSetWalker::new(
            self.engine.schema.walker(names),
            self.plan,
            self.plan_id,
            self.plan.execution_plans[self.plan_id].root.id,
        )
    }
}
