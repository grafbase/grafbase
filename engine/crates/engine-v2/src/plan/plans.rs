use super::{
    planner::{Planner, ToBePlanned},
    ExecutionPlan, ExecutionPlanRoot, PlanId,
};
use crate::{
    request::{Operation, QueryPath},
    Engine,
};
use std::{
    collections::VecDeque,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};

pub struct ExecutionPlans {
    plans: Vec<ExecutionPlan>,              // nodes
    parent_to_child: Vec<(PlanId, PlanId)>, // outgoing edges (sorted by parent)
    unexecuted_parent_count: Vec<AtomicUsize>,
    total_executed_count: AtomicUsize,
}

impl ExecutionPlans {
    pub fn initialize(engine: &Engine, operation: &Operation) -> Self {
        let mut plans = Self {
            plans: vec![],
            parent_to_child: vec![],
            unexecuted_parent_count: vec![],
            total_executed_count: AtomicUsize::new(0),
        };
        let to_be_planned = VecDeque::from([ToBePlanned {
            parent: None,
            object_id: operation.root_object_id,
            root: ExecutionPlanRoot {
                path: QueryPath::empty(),
                merged_selection_set_ids: vec![operation.root_selection_set_id],
            },
        }]);
        let mut planner = Planner {
            engine,
            operation,
            plans: &mut plans,
            to_be_planned,
        };
        while let Some(to_be_planned) = planner.to_be_planned.pop_front() {
            planner.plan_fields(to_be_planned);
        }
        plans
    }

    pub(super) fn push(&mut self, plan: ExecutionPlan) -> PlanId {
        let id = PlanId::from(self.plans.len());
        self.plans.push(plan);
        self.unexecuted_parent_count.push(AtomicUsize::new(0));
        id
    }

    pub(super) fn add_dependency(&mut self, child: PlanId, parent: PlanId) {
        self.parent_to_child.push((parent, child));
        self.unexecuted_parent_count[usize::from(child)].fetch_add(1, Relaxed);
    }

    pub fn all_without_dependencies(&self) -> Vec<PlanId> {
        self.unexecuted_parent_count
            .iter()
            .enumerate()
            .filter_map(|(plan_id, in_degree)| {
                if in_degree.load(Relaxed) == 0 {
                    Some(PlanId::from(plan_id))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn next_executable(&self, plan_id: PlanId) -> Vec<PlanId> {
        self.total_executed_count.fetch_add(1, Relaxed);
        let start = self
            .parent_to_child
            .partition_point(|(parent_id, _)| *parent_id < plan_id);
        let mut executable_plan_ids = vec![];
        for &(parent_id, child_id) in &self.parent_to_child[start..] {
            if parent_id != plan_id {
                break;
            }
            self.unexecuted_parent_count[usize::from(child_id)].fetch_sub(1, Relaxed);
            if self.unexecuted_parent_count[usize::from(child_id)].load(Relaxed) == 0 {
                executable_plan_ids.push(child_id);
            }
        }
        executable_plan_ids
    }

    pub fn are_all_executed(&self) -> bool {
        self.total_executed_count.load(Relaxed) == self.unexecuted_parent_count.len()
    }
}

impl std::ops::Index<PlanId> for ExecutionPlans {
    type Output = ExecutionPlan;

    fn index(&self, id: PlanId) -> &Self::Output {
        &self.plans[usize::from(id)]
    }
}
