use super::PlanId;

pub struct ExecutableTracker {
    pub(super) parent_to_child: Vec<(PlanId, PlanId)>, // outgoing edges (sorted by parent)
    pub(super) parent_count: Vec<usize>,               // in-degree
    pub(super) executed_count: usize,
}

impl ExecutableTracker {
    pub fn all_without_dependencies(&self) -> Vec<PlanId> {
        self.parent_count
            .iter()
            .enumerate()
            .filter_map(
                |(plan_id, &in_degree)| {
                    if in_degree == 0 {
                        Some(PlanId(plan_id))
                    } else {
                        None
                    }
                },
            )
            .collect()
    }

    pub fn next_executable(&mut self, plan_id: PlanId) -> Vec<PlanId> {
        self.executed_count += 1;
        let start = self
            .parent_to_child
            .partition_point(|(parent_id, _)| *parent_id < plan_id);
        let mut executable_plan_ids = vec![];
        for &(parent_id, child_id) in &self.parent_to_child[start..] {
            if parent_id != plan_id {
                break;
            }
            self.parent_count[child_id.0] -= 1;
            if self.parent_count[child_id.0] == 0 {
                executable_plan_ids.push(child_id);
            }
        }
        executable_plan_ids
    }

    pub fn are_all_executed(&self) -> bool {
        self.executed_count == self.parent_count.len()
    }
}
