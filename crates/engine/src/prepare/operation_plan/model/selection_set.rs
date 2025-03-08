use crate::prepare::PartitionSelectionSetRecord;

use super::{OperationPlanContext, SubgraphField};

#[derive(Clone, Copy)]
pub(crate) struct SubgraphSelectionSet<'a> {
    pub(in crate::prepare::operation_plan::model) ctx: OperationPlanContext<'a>,
    pub(in crate::prepare::operation_plan::model) item: PartitionSelectionSetRecord,
    pub(in crate::prepare::operation_plan::model) requires_typename: bool,
}

#[allow(unused)]
impl<'a> SubgraphSelectionSet<'a> {
    pub(crate) fn fields(&self) -> impl Iterator<Item = SubgraphField<'a>> + 'a {
        self.fields_ordered_by_type_condition_then_key()
    }

    pub(crate) fn fields_ordered_by_type_condition_then_key(&self) -> impl Iterator<Item = SubgraphField<'a>> + 'a {
        let ctx = self.ctx;
        self.item
            .data_field_ids_ordered_by_type_conditions_then_key
            .into_iter()
            .filter(|id| self.ctx.plan.query_modifications.included_subgraph_request_data_fields[*id])
            .map(move |id| SubgraphField { ctx, id })
    }

    pub(crate) fn requires_typename(&self) -> bool {
        self.requires_typename
    }
}

impl std::fmt::Debug for SubgraphSelectionSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionSet")
            .field("fields", &self.fields().collect::<Vec<_>>())
            .field("requiress_typename", &self.requires_typename())
            .finish()
    }
}
