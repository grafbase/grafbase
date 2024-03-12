use std::num::NonZeroU16;

use crate::sources::Plan;

use super::{CollectedField, CollectedSelectionSet, ConditionalField, ConditionalSelectionSet, OperationPlan};

id_newtypes::U16! {
    OperationPlan.plans[PlanId] => Plan | unless "Too many execution plans",
    OperationPlan.conditional_fields[ConditionalFieldId] => ConditionalField | unless "Too many expected fields",
    OperationPlan.conditional_selection_sets[ConditionalSelectionSetId] => ConditionalSelectionSet | unless "Too many ungrouped selection sets",
    OperationPlan.collected_selection_sets[CollectedSelectionSetId] => CollectedSelectionSet | unless "Too many grouped selection sets",
    OperationPlan.collected_fields[CollectedFieldId] => CollectedField | unless "Too many concrete fields",
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
