use super::{ExecutableTracker, ExecutionPlan};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct PlanId(pub(super) usize);

pub struct ExecutionPlans {
    plans: Vec<ExecutionPlan>,              // nodes
    parent_to_child: Vec<(PlanId, PlanId)>, // outgoing edges (sorted by parent)
    parent_count: Vec<usize>,
}

impl ExecutionPlans {
    pub fn builder() -> ExecutionPlansBuilder {
        ExecutionPlansBuilder {
            plans: Vec::new(),
            parent_to_child: Vec::new(),
            parent_count: Vec::new(),
        }
    }

    pub fn build_tracker(&self) -> ExecutableTracker {
        ExecutableTracker {
            parent_to_child: self.parent_to_child.clone(),
            parent_count: self.parent_count.clone(),
            executed_count: 0,
        }
    }
}

impl std::ops::Index<PlanId> for ExecutionPlans {
    type Output = ExecutionPlan;

    fn index(&self, index: PlanId) -> &Self::Output {
        &self.plans[index.0]
    }
}

pub struct ExecutionPlansBuilder {
    plans: Vec<ExecutionPlan>,              // nodes
    parent_to_child: Vec<(PlanId, PlanId)>, // outgoing edges
    parent_count: Vec<usize>,               // in-degree
}

impl ExecutionPlansBuilder {
    pub fn build(mut self) -> ExecutionPlans {
        self.parent_to_child.sort_unstable();
        ExecutionPlans {
            plans: self.plans,
            parent_to_child: self.parent_to_child,
            parent_count: self.parent_count,
        }
    }

    pub fn push(&mut self, plan: ExecutionPlan) -> PlanId {
        self.plans.push(plan);
        self.parent_count.push(0);
        PlanId(self.plans.len() - 1)
    }

    pub fn add_dependency(&mut self, child: PlanId, parent: PlanId) {
        self.parent_to_child.push((parent, child));
        self.parent_count[child.0] += 1;
    }
}

impl std::ops::Index<PlanId> for ExecutionPlansBuilder {
    type Output = ExecutionPlan;

    fn index(&self, index: PlanId) -> &Self::Output {
        &self.plans[index.0]
    }
}
