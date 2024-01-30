use std::num::NonZeroU16;

use crate::sources::ExecutionPlan;

use super::{
    ConcreteField, ConcreteSelectionSet, ConditionalField, ConditionalSelectionSet, LogicalPlan, OperationPlan,
};

crate::utils::id_newtypes! {
    OperationPlan.plans[PlanId] => LogicalPlan unless "Too many plans",
    OperationPlan.execution_plans[ExecutionPlanId] => ExecutionPlan unless "Too many execution plans",
    OperationPlan.collected_conditional_fields[ConditionalFieldId] => ConditionalField unless "Too many expected fields",
    OperationPlan.collected_conditional_selection_sets[ConditionalSelectionSetId] => ConditionalSelectionSet unless "Too many ungrouped selection sets",
    OperationPlan.collected_concrete_selection_sets[ConcreteSelectionSetId] => ConcreteSelectionSet unless "Too many grouped selection sets",
    OperationPlan.collected_concrete_fields[ConcreteFieldId] => ConcreteField unless "Too many concrete fields",
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct PlanBoundaryId(NonZeroU16);

impl From<PlanBoundaryId> for usize {
    fn from(id: PlanBoundaryId) -> usize {
        (id.0.get() - 1) as usize
    }
}

impl From<usize> for PlanBoundaryId {
    fn from(value: usize) -> Self {
        Self(
            u16::try_from(value)
                .ok()
                .and_then(|value| NonZeroU16::new(value + 1))
                .expect("Too many plan boundaries"),
        )
    }
}
