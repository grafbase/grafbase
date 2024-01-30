use schema::FieldId;
use std::borrow::Borrow;

use fnv::FnvHashMap;

use crate::{
    request::{BoundFieldId, BoundSelectionSetId, OperationWalker},
    response::{ResponseEdge, ResponseKey},
};

#[derive(Debug)]
pub(super) struct GroupForResponseKey<Item> {
    pub edge: ResponseEdge,
    pub final_bound_field_id: BoundFieldId,
    pub items: Vec<Item>,
    pub subselection_set_ids: Vec<BoundSelectionSetId>,
}

impl<'a> OperationWalker<'a> {
    pub(super) fn group_by_response_key<Item: Borrow<BoundFieldId>>(
        &self,
        items: impl IntoIterator<Item = Item>,
    ) -> FnvHashMap<ResponseKey, GroupForResponseKey<Item>> {
        let operation = self.as_ref();
        items.into_iter().fold(Default::default(), |mut groups, item| {
            let id = *item.borrow();
            let field = &operation[id];
            let edge = field.response_edge();
            let group = groups
                .entry(field.response_key())
                .or_insert_with(|| GroupForResponseKey {
                    edge,
                    final_bound_field_id: id,
                    items: Vec::new(),
                    subselection_set_ids: Vec::new(),
                });
            if edge < group.edge {
                group.edge = edge;
                group.final_bound_field_id = id;
            }
            group.items.push(item);
            if let Some(id) = operation[id].selection_set_id() {
                group.subselection_set_ids.push(id);
            }

            groups
        })
    }
}

#[derive(Debug)]
pub(super) struct GroupedByFieldId {
    pub final_bound_field_id: BoundFieldId,
    pub edge: ResponseEdge,
    pub bound_field_ids: Vec<BoundFieldId>,
    pub subselection_set_ids: Vec<BoundSelectionSetId>,
}

impl<'a> OperationWalker<'a> {
    pub(super) fn group_by_schema_field_id<Item: Borrow<BoundFieldId>>(
        &self,
        fields: impl IntoIterator<Item = Item>,
    ) -> FnvHashMap<FieldId, GroupedByFieldId> {
        let operation = self.as_ref();
        fields.into_iter().fold(Default::default(), |mut map, id| {
            let id = *id.borrow();
            let bound_field = &operation[id];
            if let Some(field_id) = bound_field.schema_field_id() {
                let group = map.entry(field_id).or_insert_with(|| GroupedByFieldId {
                    final_bound_field_id: id,
                    edge: bound_field.response_edge(),
                    bound_field_ids: Vec::new(),
                    subselection_set_ids: Vec::new(),
                });
                let edge = bound_field.response_edge();
                if edge < group.edge {
                    group.edge = edge;
                    group.final_bound_field_id = id
                }
                group.bound_field_ids.push(id);
                if let Some(id) = bound_field.selection_set_id() {
                    group.subselection_set_ids.push(id)
                }
            }
            map
        })
    }
}
