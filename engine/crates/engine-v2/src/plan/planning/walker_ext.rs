use schema::FieldId;

use std::collections::HashMap;

use crate::{
    request::{BoundFieldId, OperationWalker},
    response::ResponseKey,
};

impl<'a> OperationWalker<'a> {
    /// Sorting is used to ensure we always pick the BoundFieldId with the lowest query position.
    pub(super) fn group_by_response_key_sorted_by_query_position(
        &self,
        values: impl IntoIterator<Item = BoundFieldId>,
    ) -> HashMap<ResponseKey, Vec<BoundFieldId>> {
        let operation = self.as_ref();
        let mut grouped: HashMap<ResponseKey, Vec<BoundFieldId>> =
            values.into_iter().fold(Default::default(), |mut groups, id| {
                let field = &operation[id];
                groups.entry(field.response_key()).or_default().push(id);
                groups
            });
        for group in grouped.values_mut() {
            group.sort_unstable_by_key(|id| operation[*id].query_position())
        }
        grouped
    }
}

impl<'a> OperationWalker<'a> {
    /// Sorting is used to ensure we always pick the BoundFieldId with the lowest query position.
    pub(super) fn group_by_schema_field_id_sorted_by_query_position(
        &self,
        values: impl IntoIterator<Item = BoundFieldId>,
    ) -> HashMap<FieldId, Vec<BoundFieldId>> {
        let operation = self.as_ref();
        let mut grouped: HashMap<FieldId, Vec<BoundFieldId>> =
            values.into_iter().fold(Default::default(), |mut groups, id| {
                let bound_field = &operation[id];
                if let Some(field_id) = bound_field.schema_field_id() {
                    groups.entry(field_id).or_default().push(id);
                }
                groups
            });
        for group in grouped.values_mut() {
            group.sort_unstable_by_key(|id| operation[*id].query_position())
        }
        grouped
    }
}
