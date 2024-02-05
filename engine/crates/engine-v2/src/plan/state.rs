use std::sync::Arc;

use schema::Schema;

use crate::request::FlatTypeCondition;
use crate::response::{ResponseBoundaryItem, ResponseBuilder};

use crate::plan::{OperationPlan, PlanBoundaryId};

use super::PlanId;

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
    /// PlanBoundaryId -> u8
    plan_boundary_consummers_count: Vec<u8>,
    /// PlanBoundaryId -> Option<BoundaryItems>
    boundaries: Vec<Option<BoundaryItems>>,
}

#[derive(Clone)]
struct BoundaryItems {
    items: Arc<Vec<ResponseBoundaryItem>>,
    consummers_left: u8,
}

impl OperationExecutionState {
    pub(super) fn new(operation: &OperationPlan) -> Self {
        Self {
            plan_dependencies_count: operation.plan_dependencies_count.clone(),
            plan_boundary_consummers_count: operation.plan_boundary_consummers_count.clone(),
            boundaries: vec![None; operation.plan_boundary_consummers_count.len()],
        }
    }

    pub fn pop_subscription_plan_id(&mut self) -> PlanId {
        let executable = self.get_executable_plans();
        assert!(executable.len() == 1);
        let plan_id = executable[0];
        // Ensuring we never schedule it
        self.plan_dependencies_count[usize::from(plan_id)] = u8::MAX;
        plan_id
    }

    pub fn get_executable_plans(&self) -> Vec<PlanId> {
        self.plan_dependencies_count
            .iter()
            .enumerate()
            .filter_map(|(i, &count)| if count == 0 { Some(PlanId::from(i)) } else { None })
            .collect()
    }

    pub fn push_boundary_items(&mut self, boundary_id: PlanBoundaryId, items: Vec<ResponseBoundaryItem>) {
        self.boundaries[usize::from(boundary_id)] = Some(BoundaryItems {
            items: Arc::new(items),
            consummers_left: self.plan_boundary_consummers_count[usize::from(boundary_id)],
        });
    }

    pub fn retrieve_boundary_items(
        &mut self,
        schema: &Schema,
        operation: &OperationPlan,
        response: &ResponseBuilder,
        plan_id: PlanId,
    ) -> Arc<Vec<ResponseBoundaryItem>> {
        // If there is no root, an error propagated up to it and data will be null. So there's
        // nothing to do anymore.
        let Some(root_boundary_item) = response.root_response_boundary_item() else {
            return Arc::new(Vec::new());
        };
        let Some(input) = &operation.plan_inputs[usize::from(plan_id)] else {
            return Arc::new(vec![root_boundary_item]);
        };
        let items = {
            let n = usize::from(input.boundary_id);
            let Some(ref mut boundary) = self.boundaries[n] else {
                unreachable!("Missing boundary items");
            };
            boundary.consummers_left -= 1;
            if boundary.consummers_left == 0 {
                let items = boundary.items.clone();
                self.boundaries[n] = None;
                items
            } else {
                boundary.items.clone()
            }
        };
        match &operation.plan_outputs[usize::from(plan_id)].type_condition {
            Some(FlatTypeCondition::Interface(id)) => {
                let possible_types = &schema[*id].possible_types;
                Arc::new(
                    items
                        .iter()
                        .filter(|root| possible_types.binary_search(&root.object_id).is_ok())
                        .cloned()
                        .collect(),
                )
            }
            Some(FlatTypeCondition::Objects(ids)) if ids.len() == 1 => {
                let id = ids[0];
                Arc::new(items.iter().filter(|root| root.object_id == id).cloned().collect())
            }
            Some(FlatTypeCondition::Objects(ids)) => Arc::new(
                items
                    .iter()
                    .filter(|root| ids.binary_search(&root.object_id).is_ok())
                    .cloned()
                    .collect(),
            ),
            None => items,
        }
    }

    pub fn get_next_executable_plans(&mut self, operation: &OperationPlan, plan_id: PlanId) -> Vec<PlanId> {
        let edges = &operation.plan_parent_to_child_edges;
        let mut executable = Vec::new();
        let mut i = edges.partition_point(|edge| edge.parent < plan_id);
        while i < edges.len() && edges[i].parent == plan_id {
            let child = edges[i].child;
            let j = usize::from(child);
            self.plan_dependencies_count[j] -= 1;
            tracing::debug!(
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
