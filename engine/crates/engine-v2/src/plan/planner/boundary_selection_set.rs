use std::collections::HashMap;

use schema::{FieldId, ResolverId};

use crate::{
    plan::{attribution::ExtraField, PlanId},
    request::{FlatSelectionSetId, GroupForFieldId, OperationWalker, SelectionSetType},
};

/// Currently Operation is immutable during the planning phase. That's something I need to fix,
/// but in the meantime it is what is.
///
/// When planning children we need to keep track of which fields are present and which ones have
/// been added to ensures that we don't add more extra fields than necessary (on different plans or
/// duplciated).
///
/// So this struct represents the selection set at the boundary including the extra fields. It
/// won't contain everything though. It's initialised with the providable fields of the parent plan
/// if any and then extended during the planning with any fields that were required by children.
/// Ensuring any extra field added or providable field by one child is visible to the others
#[derive(Debug)]
pub(super) struct BoundarySelectionSet {
    pub id: FlatSelectionSetId,
    pub fields: HashMap<FieldId, BoundaryField>,
}

#[derive(Debug)]
pub(super) enum BoundaryField {
    // Field planned by either the parent plan or a child.
    Planned(PlannedBoundaryField),
    // Extra field required by a child.
    Extra {
        plan_id: PlanId,
        resolver_id: ResolverId,
        field: ExtraBoundaryField,
    },
}

#[derive(Debug)]
pub(super) struct PlannedBoundaryField {
    pub(super) plan_id: PlanId,
    pub(super) field: GroupForFieldId,
    lazy_subselection: Option<BoundarySelectionSet>,
}

impl PlannedBoundaryField {
    pub(super) fn new(plan_id: PlanId, field: GroupForFieldId) -> Self {
        Self {
            plan_id,
            field,
            lazy_subselection: None,
        }
    }

    pub(super) fn subselection_mut(&mut self, walker: OperationWalker<'_>) -> Option<&mut BoundarySelectionSet> {
        if self.field.bound_field_ids.is_empty() {
            return None;
        }
        Some(self.lazy_subselection.get_or_insert_with(|| {
            let flat_selection_set = walker.merged_selection_sets(&self.field.bound_field_ids);
            let id = flat_selection_set.id();
            let fields = flat_selection_set
                .group_by_field_id()
                .into_iter()
                .map(|(field_id, field)| {
                    (
                        field_id,
                        BoundaryField::Planned(Self {
                            plan_id: self.plan_id,
                            field,
                            lazy_subselection: None,
                        }),
                    )
                })
                .collect();
            BoundarySelectionSet { id, fields }
        }))
    }

    pub(super) fn take_subselection_if_read(self) -> Option<BoundarySelectionSet> {
        self.lazy_subselection
    }
}

#[derive(Debug, Clone)]
pub(in crate::plan) struct ExtraBoundaryField {
    pub extra_field: ExtraField<ExtraBoundarySelectionSet>,
    // Keeping track of whether an extra field is actually read by a child. We plan extra fields
    // eagerly to determine whether the full `requires` field set is completely providable or not.
    // It's only once a child plan candidate is selected and we create its input that we flag the
    // extras as 'read'. This ensures only extra fields we actually use are retrieved from
    // upstream.
    pub read: bool,
}

#[derive(Debug, Clone)]
pub(in crate::plan) struct ExtraBoundarySelectionSet {
    pub ty: SelectionSetType,
    pub fields: HashMap<FieldId, ExtraBoundaryField>,
}
