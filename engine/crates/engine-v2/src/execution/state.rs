use std::sync::Arc;

use id_derives::IndexImpls;
use schema::Schema;

use crate::response::{InputdResponseObjectSet, ResponseBuilder, ResponseObjectSet, ResponseObjectSetId};

use super::{ExecutableOperation, ExecutionPlanId, ResponseModifierExecutorId};

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
#[derive(Clone, IndexImpls)]
pub(crate) struct OperationExecutionState<'ctx> {
    schema: &'ctx Schema,
    operation: &'ctx ExecutableOperation,
    #[indexed_by(ResponseObjectSetId)]
    response_object_sets: Vec<Option<Arc<ResponseObjectSet>>>,
    #[indexed_by(ExecutionPlanId)]
    execution_plan_to_parent_count: Vec<usize>,
    #[indexed_by(ResponseModifierExecutorId)]
    response_modifier_executor_to_parent_count: Vec<usize>,
}

impl<'ctx> OperationExecutionState<'ctx> {
    pub(super) fn new(schema: &'ctx Schema, operation: &'ctx ExecutableOperation) -> Self {
        Self {
            schema,
            operation,
            response_object_sets: vec![None; operation.response_blueprint.response_object_sets_to_type.len()],
            execution_plan_to_parent_count: operation.execution_plans.iter().map(|plan| plan.parent_count).collect(),
            response_modifier_executor_to_parent_count: operation
                .response_modifier_executors
                .iter()
                .map(|exec| exec.parent_count)
                .collect(),
        }
    }

    pub fn pop_subscription_plan_id(&mut self) -> ExecutionPlanId {
        let executable = self.get_executable_plans();
        assert!(executable.len() == 1);
        let plan_id = executable[0];
        // Ensuring we never schedule it
        self[plan_id] = usize::MAX;
        plan_id
    }

    pub fn get_executable_plans(&self) -> Vec<ExecutionPlanId> {
        self.execution_plan_to_parent_count
            .iter()
            .enumerate()
            .filter_map(|(i, &count)| {
                if count == 0 {
                    Some(ExecutionPlanId::from(i))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn push_response_objects(&mut self, set_id: ResponseObjectSetId, response_object_refs: ResponseObjectSet) {
        tracing::trace!("Pushing response objects for {set_id}: {}", response_object_refs.len());
        self[set_id] = Some(Arc::new(response_object_refs));
    }

    pub fn get_input(&mut self, response: &ResponseBuilder, plan_id: ExecutionPlanId) -> InputdResponseObjectSet {
        // If there is no root, an error propagated up to it and data will be null. So there's
        // nothing to do anymore.
        let Some(root_ref) = response.root_response_object() else {
            return Default::default();
        };
        let logical_plan_id = self.operation[plan_id].logical_plan_id;
        let input_id = self.operation.response_blueprint[logical_plan_id].input_id;
        tracing::trace!("Get response objects for {input_id}");

        let output = InputdResponseObjectSet::default();
        if let Some(refs) = &self[input_id] {
            output.with_filtered_response_objects(
                self.schema,
                self.operation[logical_plan_id].entity_id,
                Arc::clone(refs),
            )
        } else if usize::from(input_id) == 0 {
            output.with_response_objects(Arc::new(vec![root_ref]))
        } else {
            output
        }
    }

    /// We just finished a plan, which response modifiers should be executed next?
    pub fn get_next_executable_response_modifiers(
        &mut self,
        plan_id: ExecutionPlanId,
    ) -> Vec<ResponseModifierExecutorId> {
        let mut executable = Vec::new();
        for child in self.operation[plan_id].dependent_response_modifiers.iter().copied() {
            self[child] -= 1;
            tracing::trace!(
                "Response modifier executor {child} has {} dependencies left",
                self[child],
            );
            if self[child] == 0 {
                executable.push(child);
            }
        }
        executable
    }

    /// We just finished a plan and applied all the relevant response modifiers, which plans should
    /// be executed next?
    pub fn get_next_executable_plans(
        &mut self,
        plan_id: ExecutionPlanId,
        response_modifier_executor_ids: Vec<ResponseModifierExecutorId>,
    ) -> Vec<ExecutionPlanId> {
        let mut executable = Vec::new();
        for &child in &self.operation[plan_id].children {
            self[child] -= 1;
            tracing::trace!("Child plan {child} has {} dependencies left", self[child],);
            if self[child] == 0 {
                executable.push(child);
            }
        }
        for response_modifier_executor_id in response_modifier_executor_ids {
            let response_modifier_executor = &self.operation[response_modifier_executor_id];
            for &child in &response_modifier_executor.children {
                self[child] -= 1;
                tracing::trace!("Child plan {child} has {} dependencies left", self[child],);
                if self[child] == 0 {
                    executable.push(child);
                }
            }
        }
        executable
    }
}
