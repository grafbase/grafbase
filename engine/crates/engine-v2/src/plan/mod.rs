use schema::ResolverId;

use crate::response_graph::{InputNodeSelectionSet, NodePath, OutputNodeSelectionSet};

mod graph;
mod planner;
pub use graph::{ExecutionPlanGraph, PlanId};
pub use planner::RequestPlan;

pub struct ExecutionPlan {
    pub path: NodePath,
    pub input: InputNodeSelectionSet,
    pub output: OutputNodeSelectionSet,
    pub resolver_id: ResolverId,
}
