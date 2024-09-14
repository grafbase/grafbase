use std::collections::HashSet;

use schema::Schema;

use crate::operation::{OperationWalker, SelectionSetWalker};

#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub(crate) enum OperationLimitExceededError {
    #[error("Query is too high.")]
    QueryTooHigh,
}

/// Enforces the operation limits specified in the schema settings.
///
/// This function checks the height of the operation's selection set against
/// the configured maximum height. If the height exceeds the allowed limit,
/// an `OperationLimitExceededError` is returned.
///
/// # Arguments
///
/// * `schema` - A reference to the schema containing the operation limits.
/// * `operation` - The operation walker that provides access to the selection
///   set.
///
/// # Returns
///
/// Returns `Ok(())` if the operation does not exceed the limits, or
/// `Err(OperationLimitExceededError::QueryTooHigh)` if the limits are exceeded.
pub(super) fn enforce_operation_limits(
    schema: &Schema,
    operation: OperationWalker<'_>,
) -> Result<(), OperationLimitExceededError> {
    let operation_limits = &schema.settings.operation_limits;
    let selection_set = operation.selection_set();

    // All other limits are verified before the binding step.
    if let Some(max_height) = operation_limits.height {
        let height = selection_set.height(&mut Default::default());
        if height > max_height {
            return Err(OperationLimitExceededError::QueryTooHigh);
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
