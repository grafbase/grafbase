use schema::ResolverId;

use crate::{
    request::{BoundSelectionSetId, QueryPath},
    response::ReadSelectionSet,
};

mod attribution;
mod expectation;
mod id;
mod planner;
mod plans;
mod selection_set;

pub use attribution::Attribution;
pub use expectation::*;
pub use id::PlanId;
pub use plans::ExecutionPlans;

#[derive(Debug, Clone)]
pub struct ExecutionPlanRoot {
    pub path: QueryPath,
    pub merged_selection_set_ids: Vec<BoundSelectionSetId>,
}

// the actual selection_set that will be resolved is determined at runtime after @skip/@include
// have been computed.
pub struct ExecutionPlan {
    pub root: ExecutionPlanRoot,
    pub input: ReadSelectionSet,
    pub resolver_id: ResolverId,
    pub attribution: Attribution,
    pub expectation: ExpectedSelectionSet,
}
