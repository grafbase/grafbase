use schema::ResolverId;

use crate::{
    request::{BoundSelectionSetId, Operation, QueryPath},
    response::ReadSelectionSet,
    Engine,
};

mod attribution;
mod id;
mod planner;
mod plans;
mod tracker;
mod walkers;

pub use attribution::Attribution;
pub use id::PlanId;
pub use planner::{PrepareError, PrepareResult};
pub use plans::ExecutionPlans;
pub use tracker::ExecutionPlansTracker;
pub use walkers::*;

#[derive(Debug, Clone)]
pub struct SelectionSetRoot {
    pub path: QueryPath,
    pub id: BoundSelectionSetId,
}

// the actual selection_set that will be resolved is determined at runtime after @skip/@include
// have been computed.
pub struct ExecutionPlan {
    pub root: SelectionSetRoot,
    pub input: ReadSelectionSet,
    pub resolver_id: ResolverId,
}

// This is the part that should be cached for a GraphQL query.
pub struct OperationPlan {
    pub operation: Operation,
    pub execution_plans: ExecutionPlans,
    pub attribution: Attribution,
    pub final_read_selection_set: ReadSelectionSet,
}

impl OperationPlan {
    pub fn prepare(engine: &Engine, operation: Operation) -> PrepareResult<OperationPlan> {
        planner::plan_operation(engine, operation)
    }
}
