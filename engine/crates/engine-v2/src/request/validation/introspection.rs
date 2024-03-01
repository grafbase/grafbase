use schema::Schema;

use crate::request::{BoundSelectionSetWalker, Location, Operation};

use super::ValidationError;

pub(super) fn ensure_introspection_is_accepted(
    schema: &Schema,
    operation: &Operation,
    request: &engine::Request,
) -> Result<(), ValidationError> {
    if operation.is_query() {
        let selection_set = operation.walk_selection_set(schema.walker());

        match request.introspection_state() {
            engine::IntrospectionState::ForceEnabled => {}
            engine::IntrospectionState::ForceDisabled => detect_introspection(selection_set)?,
            engine::IntrospectionState::UserPreference => {
                if schema.disable_introspection {
                    detect_introspection(selection_set)?;
                }
            }
        };
    }

    Ok(())
}

fn detect_introspection(selection_set: BoundSelectionSetWalker<'_>) -> Result<(), ValidationError> {
    if let Some(location) = selection_set.find_introspection_field_location() {
        return Err(ValidationError::IntrospectionWhenDisabled { location });
    }
    Ok(())
}

impl BoundSelectionSetWalker<'_> {
    fn find_introspection_field_location(self) -> Option<Location> {
        self.fields().find_map(|field| {
            let schema_field = field.schema_field();
            if schema_field.is_some_and(|field| field.name() == "__type" || field.name() == "__schema") {
                field.name_location()
            } else {
                None
            }
        })
    }
}
