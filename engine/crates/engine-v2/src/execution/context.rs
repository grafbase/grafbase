use schema::Names;

use super::{
    walkers::{SelectionSetWalker, VariablesWalker, WalkerContext},
    Variables,
};
use crate::{
    plan::{ExecutionPlan, PlanId},
    request::Operation,
    response::{ResponseObjectRoot, ResponseObjectWriter, ResponsePartBuilder},
    Engine,
};

/// Data available during the executor life during its build & execution phases.
#[derive(Clone)]
pub struct ExecutionContext<'ctx, 'names> {
    pub engine: &'ctx Engine,
    pub names: &'names dyn Names,
    pub operation: &'ctx Operation,
    pub plan_id: PlanId,
    pub(super) plan: &'ctx ExecutionPlan,
    pub(super) variables: &'ctx Variables<'ctx>,
}

impl<'ctx, 'names> ExecutionContext<'ctx, 'names> {
    // Not exactly how to handle Names overall, the Planner already needs it so we could provide it
    // to the context directly.
    #[allow(dead_code)]
    fn with_names<'other>(self, names: &'other dyn Names) -> ExecutionContext<'ctx, 'other> {
        ExecutionContext { names, ..self }
    }

    /// If you do no need to rename anything, use this walker with the schema names.
    pub fn selection_set(&self) -> SelectionSetWalker<'names>
    where
        'ctx: 'names,
    {
        WalkerContext {
            schema_walker: self.engine.schema.walker(self.names),
            operation: self.operation,
            attribution: &self.plan.attribution,
            variables: self.variables,
        }
        .walk_selection_set(self.plan.root.merged_selection_set_ids.clone())
    }

    pub fn variables(&self) -> VariablesWalker<'names>
    where
        'ctx: 'names,
    {
        VariablesWalker::new(self.engine.schema.walker(self.names), self.variables)
    }

    pub fn writer<'w>(
        &self,
        data_part: &'w mut ResponsePartBuilder,
        root: ResponseObjectRoot,
    ) -> ResponseObjectWriter<'w>
    where
        'ctx: 'w,
        'names: 'w,
    {
        ResponseObjectWriter::new(
            self.engine.schema.walker(self.names),
            self.operation,
            self.variables,
            data_part,
            root,
            &self.plan.expectation,
        )
    }
}
