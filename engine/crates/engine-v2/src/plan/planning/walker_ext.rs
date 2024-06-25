use std::collections::HashMap;

use crate::{
    operation::{FieldId, OperationWalker},
    response::ResponseKey,
};

impl<'a> OperationWalker<'a, (), ()> {
    /// Sorting is used to ensure we always pick the BoundFieldId with the lowest query position.
    pub(crate) fn group_by_response_key_sorted_by_query_position(
        &self,
        values: impl IntoIterator<Item = FieldId>,
    ) -> HashMap<ResponseKey, Vec<FieldId>> {
        let operation = self.as_ref();
        let mut grouped: HashMap<ResponseKey, Vec<FieldId>> =
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
