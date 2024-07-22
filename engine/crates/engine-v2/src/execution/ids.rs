use crate::response::GraphqlError;

use super::{ExecutableOperation, ExecutionPlan, ResponseModifierExecutor};

id_newtypes::NonZeroU16! {
    ExecutableOperation.execution_plans[ExecutionPlanId] => ExecutionPlan,
    ExecutableOperation.query_modifications.errors[ErrorId] => GraphqlError,
    ExecutableOperation.response_modifier_executors[ResponseModifierExecutorId] => ResponseModifierExecutor,
}
