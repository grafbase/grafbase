use std::collections::HashSet;

use crate::request::{BoundFieldId, BoundSelectionSetId};

pub struct Attribution {
    selection_sets: Vec<BoundSelectionSetId>,
    fields: Vec<BoundFieldId>,
}

impl Attribution {
    pub fn field(&self, id: BoundFieldId) -> bool {
        self.fields.binary_search(&id).is_ok()
    }

    pub fn selection_set(&self, id: BoundSelectionSetId) -> bool {
        self.selection_sets.binary_search(&id).is_ok()
    }
}

pub(super) struct AttributionBuilder {
    pub(super) selection_sets: HashSet<BoundSelectionSetId>,
    pub(super) fields: Vec<BoundFieldId>,
}

impl AttributionBuilder {
    pub(super) fn build(self) -> Attribution {
        let mut attribution = Attribution {
            selection_sets: self.selection_sets.into_iter().collect(),
            fields: self.fields,
        };
        attribution.selection_sets.sort_unstable();
        attribution.fields.sort_unstable();
        attribution
    }
}
