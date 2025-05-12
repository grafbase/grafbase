use walker::Walk as _;

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
    pub(crate) fn is_empty(&self) -> bool {
        self.item.data_field_ids_ordered_by_parent_entity_then_key.is_empty() && self.item.typename_field_ids.is_empty()
    }

    pub(crate) fn fields(&self) -> impl Iterator<Item = SubgraphField<'a>> + 'a {
        let ctx = self.ctx;
        self.data_fields_ordered_by_parent_entity_then_key().chain(
            self.item
                .lookup_field_ids
                .into_iter()
                .map(move |id| SubgraphField { ctx, id: id.into() }),
        )
    }

    pub(crate) fn data_fields_ordered_by_parent_entity_then_key(&self) -> impl Iterator<Item = SubgraphField<'a>> + 'a {
        let ctx = self.ctx;
        self.item
            .data_field_ids_ordered_by_parent_entity_then_key
            .into_iter()
            .filter(move |id| {
                ctx.plan.query_modifications.included_subgraph_request_data_fields[*id]
                    && id.walk(ctx).derived.is_none()
            })
            .map(move |id| SubgraphField { ctx, id: id.into() })
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

impl<'a> runtime::extension::SelectionSet<'a> for SubgraphSelectionSet<'a> {
    type Field = SubgraphField<'a>;
    fn requires_typename(&self) -> bool {
        self.requires_typename()
    }

    fn fields_ordered_by_parent_entity(&self) -> impl Iterator<Item = Self::Field> {
        self.data_fields_ordered_by_parent_entity_then_key()
    }
}

impl<'a> runtime::extension::DynSelectionSet<'a> for SubgraphSelectionSet<'a> {
    fn requires_typename(&self) -> bool {
        self.requires_typename()
    }
    fn fields_ordered_by_parent_entity(&self) -> Vec<Box<dyn runtime::extension::DynField<'a>>> {
        self.data_fields_ordered_by_parent_entity_then_key()
            .map(|field| -> Box<dyn runtime::extension::DynField<'a>> { Box::new(field) })
            .collect()
    }
}
