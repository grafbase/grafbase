use std::num::NonZeroU16;

use super::{
    CollectedField, CollectedSelectionSet, ConditionalField, ConditionalSelectionSet, ExecutionPlan, OperationPlan,
};

id_newtypes::NonZeroU16! {
    OperationPlan.execution_plans[ExecutionPlanId] => ExecutionPlan,
    OperationPlan.conditional_fields[ConditionalFieldId] => ConditionalField,
    OperationPlan.conditional_selection_sets[ConditionalSelectionSetId] => ConditionalSelectionSet,
    OperationPlan.collected_selection_sets[CollectedSelectionSetId] => CollectedSelectionSet,
    OperationPlan.collected_fields[CollectedFieldId] => CollectedField,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct ExecutionPlanBoundaryId(NonZeroU16);

impl From<ExecutionPlanBoundaryId> for usize {
    fn from(id: ExecutionPlanBoundaryId) -> usize {
        (id.0.get() - 1) as usize
    }
}

impl From<usize> for ExecutionPlanBoundaryId {
    fn from(value: usize) -> Self {
        Self(
            u16::try_from(value)
                .ok()
                .and_then(|value| NonZeroU16::new(value + 1))
                .expect("Too many plan boundaries"),
        )
    }
}
