mod coordinator;
mod executor;
mod strings;
mod variables;
mod walkers;

pub use coordinator::ExecutorCoordinator;
pub use strings::*;
pub use variables::*;

use crate::{
    plan::{OperationPlan, PlanId},
    Engine,
};

/// Data available during the executor life during its build & execution phases.
#[derive(Clone)]
pub struct ExecutionContext<'a> {
    engine: &'a Engine,
    plan: &'a OperationPlan,
    variables: &'a Variables<'a>,
    plan_id: PlanId,
}
