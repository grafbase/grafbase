use std::num::NonZeroU16;

use crate::sources::Plan;

use super::{CollectedField, CollectedSelectionSet, ConditionalField, ConditionalSelectionSet, OperationPlan};

id_newtypes::U16! {
    OperationPlan.plans[PlanId] => Plan,
    OperationPlan.conditional_fields[ConditionalFieldId] => ConditionalField,
    OperationPlan.conditional_selection_sets[ConditionalSelectionSetId] => ConditionalSelectionSet,
    OperationPlan.collected_selection_sets[CollectedSelectionSetId] => CollectedSelectionSet,
    OperationPlan.collected_fields[CollectedFieldId] => CollectedField,
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
