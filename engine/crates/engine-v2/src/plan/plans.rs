use super::{ExecutionPlan, ExecutionPlansTracker, PlanId};

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

    pub fn build_tracker(&self) -> ExecutionPlansTracker {
        ExecutionPlansTracker {
            parent_to_child: self.parent_to_child.clone(),
            parent_count: self.parent_count.clone(),
            executed_count: 0,
        }
    }
}

impl std::ops::Index<PlanId> for ExecutionPlans {
    type Output = ExecutionPlan;

    fn index(&self, id: PlanId) -> &Self::Output {
        &self.plans[usize::from(id)]
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
        let id = PlanId::from(self.plans.len());
        self.plans.push(plan);
        self.parent_count.push(0);
        id
    }

    pub fn add_dependency(&mut self, child: PlanId, parent: PlanId) {
        self.parent_to_child.push((parent, child));
        self.parent_count[usize::from(child)] += 1;
    }
}

impl std::ops::Index<PlanId> for ExecutionPlansBuilder {
    type Output = ExecutionPlan;

    fn index(&self, index: PlanId) -> &Self::Output {
        &self.plans[usize::from(index)]
    }
}
