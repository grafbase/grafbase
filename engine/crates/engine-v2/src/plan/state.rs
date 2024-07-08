use std::sync::Arc;

use schema::{EntityId, Schema};

use crate::response::{ResponseBuilder, ResponseObjectRef};

use crate::plan::OperationPlan;

use super::{ExecutionPlanId, ResponseObjectSetId};

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
pub struct OperationExecutionState {
    /// PlanId -> u8
    plan_dependencies_count: Vec<u8>,
    /// EntityLocation -> u8
    set_consummers_count: Vec<u8>,
    /// EntityLocation -> Option<BoundaryItems>
    sets: Vec<Option<ResponseObjectSet>>,
}

#[derive(Clone)]
struct ResponseObjectSet {
    response_object_refs: Arc<Vec<ResponseObjectRef>>,
    consummers_left: u8,
}

impl OperationExecutionState {
    pub(super) fn new(operation: &OperationPlan) -> Self {
        Self {
            plan_dependencies_count: operation.plan_dependencies_count.clone(),
            set_consummers_count: operation.response_object_set_consummers_count.clone(),
            sets: vec![None; operation.response_object_set_consummers_count.len()],
        }
    }

    pub fn pop_subscription_plan_id(&mut self) -> ExecutionPlanId {
        let executable = self.get_executable_plans();
        assert!(executable.len() == 1);
        let plan_id = executable[0];
        // Ensuring we never schedule it
        self.plan_dependencies_count[usize::from(plan_id)] = u8::MAX;
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
        self.sets[usize::from(set_id)] = Some(ResponseObjectSet {
            response_object_refs: Arc::new(response_object_refs),
            consummers_left: self.set_consummers_count[usize::from(set_id)],
        });
    }

    pub fn get_root_response_object_refs(
        &mut self,
        schema: &Schema,
        operation: &OperationPlan,
        response: &ResponseBuilder,
        plan_id: ExecutionPlanId,
    ) -> Arc<Vec<ResponseObjectRef>> {
        // If there is no root, an error propagated up to it and data will be null. So there's
        // nothing to do anymore.
        let Some(root_ref) = response.root_response_object() else {
            return Arc::new(Vec::new());
        };
        let input = &operation[plan_id].input;
        let refs = {
            let i = usize::from(input.response_object_set_id);
            let Some(ref mut entities) = self.sets[i] else {
                if i == 0 {
                    return Arc::new(vec![root_ref]);
                }
                unreachable!("Missing entities");
            };
            entities.consummers_left -= 1;
            if entities.consummers_left == 0 {
                let refs = entities.response_object_refs.clone();
                self.sets[i] = None;
                refs
            } else {
                entities.response_object_refs.clone()
            }
        };
        // FIXME: it's not always necessary to clone the response_object_refs if it's always the
        // same entity.
        match &input.entity_id {
            EntityId::Interface(id) => {
                let possible_types = &schema[*id].possible_types;
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

    pub fn get_next_executable_plans(
        &mut self,
        operation: &OperationPlan,
        plan_id: ExecutionPlanId,
    ) -> Vec<ExecutionPlanId> {
        let edges = &operation.plan_parent_to_child_edges;
        let mut executable = Vec::new();
        let mut i = edges.partition_point(|edge| edge.parent < plan_id);
        while i < edges.len() && edges[i].parent == plan_id {
            let child = edges[i].child;
            let j = usize::from(child);
            self.plan_dependencies_count[j] -= 1;
            tracing::trace!(
                "Child plan {child} has {} dependencies left",
                self.plan_dependencies_count[j],
            );
            if self.plan_dependencies_count[j] == 0 {
                executable.push(child);
            }
            i += 1
        }
        executable
    }
}
