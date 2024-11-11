use std::collections::HashSet;

use schema::Schema;

use crate::operation::{BindError, BindResult, OperationWalker, SelectionSetWalker};

pub(super) fn enforce_operation_limits(schema: &Schema, operation: OperationWalker<'_>) -> BindResult<()> {
    let operation_limits = &schema.settings.operation_limits;
    let selection_set = operation.selection_set();

    // All other limits are verified before the binding step.
    if let Some(max_height) = operation_limits.height {
        let height = selection_set.height(&mut Default::default());
        if height > max_height {
            return Err(BindError::QueryTooHigh);
        }
    }

    Ok(())
}

impl SelectionSetWalker<'_> {
    // `None` stored in the set means `__typename`.
    fn height(&self, fields_seen: &mut HashSet<Option<schema::FieldDefinitionId>>) -> u16 {
        self.fields()
            .map(|field| {
                (if fields_seen.insert(field.as_ref().definition_id()) {
                    1
                } else {
                    0
                }) + field
                    .selection_set()
                    .map(|selection_set| selection_set.height(&mut HashSet::new()))
                    .unwrap_or_default()
            })
            .sum()
    }
}
