use itertools::Itertools;
use walker::{Iter, Walk};

use crate::operation::SelectionSetRecord;

use super::{OperationPlanContext, PlanDataField, PlanField, PlanTypenameField};

#[derive(Clone, Copy)]
pub(crate) struct PlanSelectionSet<'a> {
    pub(in crate::operation::plan::model) ctx: OperationPlanContext<'a>,
    pub(in crate::operation::plan::model) item: SelectionSetRecord,
    pub(in crate::operation::plan::model) requires_typename: bool,
}

#[allow(unused)]
impl<'a> PlanSelectionSet<'a> {
    pub(crate) fn fields(&self) -> impl Iterator<Item = PlanDataField<'a>> + 'a {
        self.fields_ordered_by_type_condition_then_position()
    }

    pub(crate) fn fields_ordered_by_type_condition_then_position(
        &self,
    ) -> impl Iterator<Item = PlanDataField<'a>> + 'a {
        let ctx = self.ctx;
        self.item
            .data_field_ids_ordered_by_parent_entity_id_then_key
            .into_iter()
            .filter(|id| !self.ctx.operation_plan.query_modifications.skipped_data_fields[*id])
            .map(move |id| PlanDataField { ctx, id })
    }

    pub(crate) fn requires_typename(&self) -> bool {
        self.requires_typename
    }
}

impl std::fmt::Debug for PlanSelectionSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionSet")
            .field("fields", &self.fields().collect::<Vec<_>>())
            .finish()
    }
}
