use std::sync::Arc;

use schema::{EntityId, Schema};

use crate::response::{ResponseBuilder, ResponseObjectRef, ResponseObjectSetId};

use super::{ExecutableOperation, ExecutionPlanId};

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
#[derive(Clone)]
pub(crate) struct OperationExecutionState<'ctx> {
    schema: &'ctx Schema,
    operation: &'ctx ExecutableOperation,
    response_object_sets: Vec<ResponseObjectSet>,
    plan_dependencies_count: Vec<usize>,
}

id_newtypes::index! {
    OperationExecutionState<'ctx>.plan_dependencies_count[ExecutionPlanId] => usize,
    OperationExecutionState<'ctx>.response_object_sets[ResponseObjectSetId] => ResponseObjectSet,
}

#[derive(Clone)]
pub(crate) struct ResponseObjectSet {
    refs: Option<Arc<Vec<ResponseObjectRef>>>,
}

impl<'ctx> OperationExecutionState<'ctx> {
    pub(super) fn new(schema: &'ctx Schema, operation: &'ctx ExecutableOperation) -> Self {
        Self {
            schema,
            operation,
            response_object_sets: vec![
                ResponseObjectSet { refs: None };
                operation.response_blueprint.response_object_set_count
            ],
            plan_dependencies_count: operation.execution_plans.iter().map(|plan| plan.parent_count).collect(),
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
        self.plan_dependencies_count
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

    pub fn push_response_objects(&mut self, set_id: ResponseObjectSetId, response_object_refs: Vec<ResponseObjectRef>) {
        self[set_id].refs = Some(Arc::new(response_object_refs));
    }

    pub fn get_root_response_object_refs(
        &mut self,
        response: &ResponseBuilder,
        plan_id: ExecutionPlanId,
    ) -> Arc<Vec<ResponseObjectRef>> {
        // If there is no root, an error propagated up to it and data will be null. So there's
        // nothing to do anymore.
        let Some(root_ref) = response.root_response_object() else {
            return Arc::new(Vec::new());
        };
        let logical_plan_id = self.operation[plan_id].logical_plan_id;
        let input_id = self.operation.response_blueprint[logical_plan_id].input_id;
        let refs = {
            let response_object_set = &mut self[input_id];
            let Some(refs) = response_object_set.refs.clone() else {
                if usize::from(input_id) == 0 {
                    return Arc::new(vec![root_ref]);
                }
                unreachable!("Missing entities");
            };
            refs
        };
        // FIXME: it's not always necessary to clone the response_object_refs if it's always the
        // same entity.
        match &self.operation[logical_plan_id].entity_id {
            EntityId::Interface(id) => {
                let possible_types = &self.schema[*id].possible_types;
                Arc::new(
                    refs.iter()
                        .filter(|obj| possible_types.binary_search(&obj.definition_id).is_ok())
                        .cloned()
                        .collect(),
                )
            }
            &EntityId::Object(id) => Arc::new(refs.iter().filter(|obj| obj.definition_id == id).cloned().collect()),
        }
    }

    pub fn get_next_executable_plans(&mut self, plan_id: ExecutionPlanId) -> Vec<ExecutionPlanId> {
        let mut executable = Vec::new();
        for child in self.operation[plan_id].children.iter().copied() {
            self[child] -= 1;
            tracing::trace!("Child plan {child} has {} dependencies left", self[child],);
            if self[child] == 0 {
                executable.push(child);
            }
        }
        executable
    }
}
