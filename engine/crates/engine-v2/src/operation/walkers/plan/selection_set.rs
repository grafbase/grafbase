use crate::operation::SelectionSetId;

use super::{PlanField, PlanWalker};

#[derive(Clone, Copy)]
pub enum PlanSelectionSet<'a> {
    RootFields(PlanWalker<'a, (), ()>),
    SelectionSet(PlanWalker<'a, SelectionSetId, ()>),
}

impl<'a> PlanSelectionSet<'a> {
    // Whether the subgraph should provide __typename (or whatever field is necessary to detect the object type)
    pub fn requires_typename(&self) -> bool {
        match self {
            PlanSelectionSet::RootFields(_) => false,
            PlanSelectionSet::SelectionSet(walker) => {
                walker.walk_with((), ()).blueprint().selection_set_to_requires_typename[walker.item]
            }
        }
    }

    pub fn fields(&self) -> Vec<PlanField<'a>> {
        self.fields_ordered_by_parent_entity_id_then_position()
    }

    pub fn fields_ordered_by_parent_entity_id_then_position(&self) -> Vec<PlanField<'a>> {
        let out = match self {
            PlanSelectionSet::RootFields(walker) => walker
                .logical_plan()
                .as_ref()
                .root_field_ids_ordered_by_parent_entity_id_then_position
                .iter()
                .filter(|id| !walker.query_modifications.skipped_fields[**id])
                .filter_map(move |&id| {
                    walker.operation[id]
                        .definition_id()
                        .map(|definition_id| walker.walk_with(id, definition_id))
                })
                .collect::<Vec<_>>(),
            PlanSelectionSet::SelectionSet(walker) => walker
                .as_ref()
                .field_ids_ordered_by_parent_entity_id_then_position
                .iter()
                .filter(|id| !walker.query_modifications.skipped_fields[**id])
                .filter_map(|id| {
                    let field_plan_id = walker.operation.plan_id_for(*id);
                    if field_plan_id == walker.logical_plan_id {
                        walker.operation[*id]
                            .definition_id()
                            .map(|definition_id| walker.walk_with(*id, definition_id))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
        };
        out
    }

    pub fn walker(&self) -> PlanWalker<'a, (), ()> {
        match self {
            PlanSelectionSet::RootFields(walker) => walker.walk_with((), ()),
            PlanSelectionSet::SelectionSet(walker) => walker.walk_with((), ()),
        }
    }
}

impl<'a> std::fmt::Debug for PlanSelectionSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanSelectionSet")
            .field("fields", &self.fields())
            .finish()
    }
}
