use crate::response::GraphqlError;

use super::{ExecutionPlan, ExecutionPlans, QueryModifications};

id_newtypes::NonZeroU16! {
    ExecutionPlans.execution_plans[ExecutionPlanId] => ExecutionPlan,
    QueryModifications.errors[ErrorId] => GraphqlError,
}
