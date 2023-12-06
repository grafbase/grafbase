use std::collections::{HashMap, HashSet};

use schema::FieldId;

use crate::{
    request::{BoundFieldId, BoundSelectionSetId},
    response::BoundResponseKey,
};

/// Keeps track of all fields associated to a certain plan. All selection_sets that have at least
/// one field, eventually nested inside a fragment, are also tracked to ensure we the plan doesn't
/// see any empty selection sets.
#[derive(Debug)]
pub struct Attribution {
    selection_sets: Vec<BoundSelectionSetId>,
    fields: Vec<BoundFieldId>,
    #[allow(unused)]
    extras: HashMap<BoundSelectionSetId, ExtraSelectionSet>,
}

#[derive(Debug)]
pub struct ExtraField {
    pub bound_response_object_id: BoundResponseKey,
    pub field_id: FieldId,
    pub selection_set: Option<ExtraSelectionSet>,
}

#[derive(Debug)]
pub struct ExtraSelectionSet {
    pub items: Vec<ExtraField>,
}

impl Attribution {
    pub fn field(&self, id: BoundFieldId) -> bool {
        self.fields.binary_search(&id).is_ok()
    }

    pub fn selection_set(&self, id: BoundSelectionSetId) -> bool {
        self.selection_sets.binary_search(&id).is_ok()
    }
}

#[derive(Default, Debug)]
pub(super) struct AttributionBuilder {
    pub selection_sets: HashSet<BoundSelectionSetId>,
    pub fields: Vec<BoundFieldId>,
    #[allow(unused)]
    pub extras: HashMap<BoundSelectionSetId, ExtraSelectionSetBuilder>,
}

#[derive(Debug)]
pub struct ExtraSelectionSetBuilder {
    pub fields: HashMap<FieldId, Option<ExtraSelectionSetBuilder>>,
}

impl AttributionBuilder {
    pub(super) fn build(self) -> Attribution {
        let mut attribution = Attribution {
            selection_sets: self.selection_sets.into_iter().collect(),
            fields: self.fields,
            extras: HashMap::with_capacity(0),
        };
        attribution.selection_sets.sort_unstable();
        attribution.fields.sort_unstable();
        attribution
    }
}
