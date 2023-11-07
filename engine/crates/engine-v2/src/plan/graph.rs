use super::ExecutionPlan;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct PlanId(usize);

pub struct ExecutionPlanGraph {
    plans: Vec<ExecutionPlan>,              // nodes
    parent_to_child: Vec<(PlanId, PlanId)>, // outgoing edges (sorted by parent)
    // Both variables underneath are changed during the execution. Need to separate them later.
    // should be a Vec<u8>, a plan having more than 255 dependencies is most likely insane
    parent_count: Vec<usize>, // in-degree
    executed_count: usize,
}

impl ExecutionPlanGraph {
    pub fn builder() -> ExecutionPlanGraphBuilder {
        ExecutionPlanGraphBuilder {
            plans: Vec::new(),
            parent_to_child: Vec::new(),
            parent_count: Vec::new(),
        }
    }

    pub fn executable_plan_ids(&self) -> Vec<PlanId> {
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

    // current used during execution, but we shouldn't. ExecutionPlanGraph should create another
    // struct having a copy of parent_count. making ExecutionPlanGraph re-usable accross executions
    // (and thus could be saved in a LRU cache)
    pub fn finished(&mut self, plan_id: PlanId) -> Vec<PlanId> {
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

    pub fn is_finished(&self) -> bool {
        self.executed_count == self.plans.len()
    }
}

impl std::ops::Index<PlanId> for ExecutionPlanGraph {
    type Output = ExecutionPlan;

    fn index(&self, index: PlanId) -> &Self::Output {
        &self.plans[index.0]
    }
}

pub struct ExecutionPlanGraphBuilder {
    plans: Vec<ExecutionPlan>,              // nodes
    parent_to_child: Vec<(PlanId, PlanId)>, // outgoing edges
    parent_count: Vec<usize>,               // in-degree
}

impl ExecutionPlanGraphBuilder {
    pub fn build(mut self) -> ExecutionPlanGraph {
        self.parent_to_child.sort_unstable();
        ExecutionPlanGraph {
            plans: self.plans,
            parent_to_child: self.parent_to_child,
            parent_count: self.parent_count,
            executed_count: 0,
        }
    }

    pub fn push_plan(&mut self, plan: ExecutionPlan) -> PlanId {
        self.plans.push(plan);
        self.parent_count.push(0);
        PlanId(self.plans.len() - 1)
    }

    pub fn push_dependency(&mut self, parent: PlanId, child: PlanId) {
        self.parent_to_child.push((parent, child));
        self.parent_count[child.0] += 1;
    }
}
