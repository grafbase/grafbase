use std::sync::Arc;

use id_derives::IndexedFields;
use walker::Walk;

use crate::{
    operation::{Executable, Plan, PlanId, ResponseModifierId, ResponseObjectSetDefinitionId},
    response::{InputResponseObjectSet, ResponseBuilder, ResponseObjectSet},
    Runtime,
};

use super::ExecutionContext;

/// Holds the current state of the operation execution:
/// - which plans have been executed
/// - boundary items between plans
///
/// It allows the `OperationPlan` to be entirely re-usable and immutable for a given request for
/// subscriptions.
///
/// Response boundary items, so objects within the response provided by one plan and updated by
/// other children plans, are also kept in this struct as long as any children plan might need
/// it.
#[derive(IndexedFields)]
pub(crate) struct OperationExecutionState<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    #[indexed_by(ResponseObjectSetDefinitionId)]
    response_object_sets: Vec<Option<Arc<ResponseObjectSet>>>,
    #[indexed_by(PlanId)]
    plan_to_parent_count: Vec<usize>,
    #[indexed_by(ResponseModifierId)]
    response_modifier_to_parent_count: Vec<usize>,
}

impl<'ctx, R: Runtime> Clone for OperationExecutionState<'ctx, R> {
    fn clone(&self) -> Self {
        Self {
            ctx: self.ctx,
            response_object_sets: self.response_object_sets.clone(),
            plan_to_parent_count: self.plan_to_parent_count.clone(),
            response_modifier_to_parent_count: self.response_modifier_to_parent_count.clone(),
        }
    }
}

impl<'ctx, R: Runtime> OperationExecutionState<'ctx, R> {
    pub(super) fn new(ctx: ExecutionContext<'ctx, R>) -> Self {
        Self {
            ctx,
            response_object_sets: vec![None; ctx.operation.cached.solved.response_object_set_definitions.len()],
            plan_to_parent_count: ctx.operation.plan.plans.iter().map(|plan| plan.parent_count).collect(),
            response_modifier_to_parent_count: ctx
                .operation
                .plan
                .response_modifiers
                .iter()
                .map(|exec| exec.parent_count)
                .collect(),
        }
    }

    pub fn pop_subscription_plan(&mut self) -> Plan<'ctx> {
        let plan = {
            let mut executable = self.get_executable_plans();
            let plan = executable.next().expect("Must have at least one plan");
            assert!(executable.next().is_none());
            plan
        };
        // Ensuring we never schedule it
        self[plan.id] = usize::MAX;
        plan
    }

    pub fn get_executable_plans(&self) -> impl Iterator<Item = Plan<'ctx>> + '_ {
        self.plan_to_parent_count.iter().enumerate().filter_map(|(i, &count)| {
            if count == 0 {
                Some(PlanId::from(i).walk(&self.ctx))
            } else {
                None
            }
        })
    }

    pub fn push_response_objects(
        &mut self,
        set_id: ResponseObjectSetDefinitionId,
        response_object_refs: ResponseObjectSet,
    ) {
        tracing::trace!("Pushing response objects for {set_id}: {}", response_object_refs.len());
        self[set_id] = Some(Arc::new(response_object_refs));
    }

    pub fn get_input(&mut self, response: &ResponseBuilder, plan: Plan<'_>) -> InputResponseObjectSet {
        // If there is no root, an error propagated up to it and data will be null. So there's
        // nothing to do anymore.
        let Some(root_ref) = response.root_response_object() else {
            return Default::default();
        };

        let input_id = plan.input_id();
        tracing::trace!("Get response objects for {input_id}");

        let output = InputResponseObjectSet::default();
        if let Some(refs) = &self[input_id] {
            output.with_filtered_response_objects(
                self.ctx.schema(),
                plan.entity_definition().id().into(),
                Arc::clone(refs),
            )
        } else if usize::from(input_id) == 0 {
            output.with_response_objects(Arc::new(vec![root_ref]))
        } else {
            output
        }
    }

    /// We just finished a plan, which response modifiers should be executed next?
    pub fn get_next_executables<'a>(&mut self, executed: impl Into<Executable<'a>>) -> Vec<Executable<'a>> {
        let executed: Executable<'_> = executed.into();

        let mut executable = Vec::new();
        for child in executed.children() {
            tracing::trace!("Child {:?}", child.id());
            match &child {
                Executable::Plan(plan) => {
                    self[plan.id] -= 1;
                    tracing::trace!("Plan {} has {} dependencies left", plan.id, self[plan.id],);
                    if self[plan.id] == 0 {
                        executable.push(child);
                    }
                }
                Executable::ResponseModifier(modifier) => {
                    self[modifier.id] -= 1;
                    tracing::trace!(
                        "Response modifier {} has {} dependencies left",
                        modifier.id,
                        self[modifier.id]
                    );
                    if self[modifier.id] == 0 {
                        executable.push(child);
                    }
                }
            }
        }
        executable
    }
}
