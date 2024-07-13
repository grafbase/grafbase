use crate::response::GraphqlError;

use super::{ExecutableOperation, ExecutionPlan};

id_newtypes::NonZeroU16! {
    ExecutableOperation.execution_plans[ExecutionPlanId] => ExecutionPlan,
    ExecutableOperation.query_modifications.errors[ErrorId] => GraphqlError,
}
