use schema::Schema;

use crate::operation::{BindError, BindResult, OperationWalker, SelectionSetWalker};

pub(super) fn ensure_introspection_is_accepted(schema: &Schema, operation: OperationWalker<'_>) -> BindResult<()> {
    if operation.is_query() && schema.settings.disable_introspection {
        detect_introspection(operation.selection_set())?;
    }

    Ok(())
}

fn detect_introspection(selection_set: SelectionSetWalker<'_>) -> BindResult<()> {
    for field in selection_set.fields() {
        if matches!(field.name(), "__schema" | "__type") {
            return Err(BindError::IntrospectionIsDisabled {
                location: field.location(),
            });
        }
    }
    Ok(())
}
