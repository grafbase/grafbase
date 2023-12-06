use crate::request::{BoundFieldId, BoundSelectionSetId};

/// Filter out fields that aren't related to a specific plan.
pub trait PlanFilter {
    fn field(&self, id: BoundFieldId) -> bool;
    fn selection_set(&self, id: BoundSelectionSetId) -> bool;
}

impl PlanFilter for () {
    fn field(&self, _id: BoundFieldId) -> bool {
        true
    }

    fn selection_set(&self, _id: BoundSelectionSetId) -> bool {
        true
    }
}
