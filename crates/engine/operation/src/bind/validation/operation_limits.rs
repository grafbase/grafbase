use std::collections::HashSet;

use schema::FieldDefinitionId;

use crate::{BindError, BindResult, OperationContext, SelectionSet};

pub(super) fn enforce_operation_limits(ctx: OperationContext<'_>) -> BindResult<()> {
    let operation_limits = &ctx.schema.settings.operation_limits;
    let selection_set = ctx.root_selection_set();

    // All other limits are verified before the binding step.
    if let Some(max_height) = operation_limits.height {
        let height = selection_set.height(&mut Default::default());
        if height > max_height {
            return Err(BindError::QueryTooHigh);
        }
    }

    Ok(())
}

impl SelectionSet<'_> {
    fn height(&self, fields_seen: &mut HashSet<FieldDefinitionId>) -> u16 {
        self.fields()
            .filter_map(|field| field.as_data())
            .map(|field| {
                (if fields_seen.insert(field.definition_id) { 1 } else { 0 })
                    + field.selection_set().height(fields_seen)
            })
            .sum()
    }
}
