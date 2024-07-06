use super::{ExecutionPlan, ExecutionPlans};

id_newtypes::NonZeroU16! {
    ExecutionPlans.execution_plans[ExecutionPlanId] => ExecutionPlan,
}
