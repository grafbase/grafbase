use schema::Schema;

use crate::operation::{OperationWalker, SelectionSetWalker};

use super::ValidationError;

pub(super) fn ensure_introspection_is_accepted(
    schema: &Schema,
    operation: OperationWalker<'_>,
    request: &engine::Request,
) -> Result<(), ValidationError> {
    if operation.is_query() {
        let selection_set = operation.selection_set();
        match request.introspection_state() {
            engine::IntrospectionState::ForceEnabled => {}
            engine::IntrospectionState::ForceDisabled => detect_introspection(selection_set)?,
            engine::IntrospectionState::UserPreference => {
                if schema.settings.disable_introspection {
                    detect_introspection(selection_set)?;
                }
            }
        };
    }

    Ok(())
}

fn detect_introspection(selection_set: SelectionSetWalker<'_>) -> Result<(), ValidationError> {
    for field in selection_set.fields() {
        if matches!(field.name(), "__schema" | "__type") {
            return Err(ValidationError::IntrospectionWhenDisabled {
                location: field.location(),
            });
        }
    }
    Ok(())
}
