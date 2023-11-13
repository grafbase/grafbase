use schema::ResolverId;

use crate::response::{ReadSelectionSet, ResponsePath, WriteSelectionSet};

mod planner;
mod plans;
pub use planner::RequestPlan;
pub use plans::{ExecutionPlans, PlanId};

pub struct ExecutionPlan {
    pub path: ResponsePath,
    pub input: ReadSelectionSet,
    pub output: WriteSelectionSet,
    pub resolver_id: ResolverId,
}
