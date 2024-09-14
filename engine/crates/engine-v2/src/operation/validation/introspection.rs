use schema::Schema;

use crate::operation::{OperationWalker, SelectionSetWalker};

use super::ValidationError;

/// Ensures that introspection is accepted based on the provided schema settings.
///
/// This function checks if introspection queries are allowed by the schema's settings.
/// If introspection is disabled and the operation is a query, it detects any
/// introspection fields in the selection set and returns a validation error.
///
/// # Parameters
///
/// - `schema`: A reference to the `Schema` which contains the settings for introspection.
/// - `operation`: The `OperationWalker` that represents the operation being validated.
///
/// # Returns
///
/// Returns a `Result` which is `Ok(())` if introspection is accepted, or a `ValidationError`
/// if introspection queries are prohibited.
pub(super) fn ensure_introspection_is_accepted(
    schema: &Schema,
    operation: OperationWalker<'_>,
) -> Result<(), ValidationError> {
    if operation.is_query() && schema.settings.disable_introspection {
        detect_introspection(operation.selection_set())?;
    }

    Ok(())
}

/// Detects if introspection fields are present in the selection set.
///
/// This function iterates through the provided selection set fields
/// and checks for the presence of introspection-specific fields,
/// namely `__schema` and `__type`.
///
/// # Parameters
///
/// - `selection_set`: The `SelectionSetWalker` containing the fields to inspect.
///
/// # Returns
///
/// - `Ok(())` if no introspection fields are found.
/// - A `ValidationError` if introspection fields are detected when
///   introspection is disabled, along with the field's location.
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
