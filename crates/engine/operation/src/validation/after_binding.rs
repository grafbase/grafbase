use std::collections::HashSet;

use schema::{FieldDefinitionId, Schema};

use crate::{Location, Operation, OperationContext, SelectionSet};

#[derive(thiserror::Error, Debug)]
pub(crate) enum ValidationError {
    #[error("Query is too high.")]
    QueryTooHigh,
    #[error("GraphQL introspection is not allowed, but the query contained __schema or __type")]
    IntrospectionIsDisabled { location: Location },
}

impl ValidationError {
    pub fn location(&self) -> Option<Location> {
        match self {
            ValidationError::QueryTooHigh => None,
            ValidationError::IntrospectionIsDisabled { location } => Some(*location),
        }
    }
}

pub(crate) fn validate(schema: &Schema, operation: &Operation) -> Result<(), Vec<ValidationError>> {
    let ctx = OperationContext { schema, operation };
    let mut errors = Vec::new();
    if let Err(err) = enforce_operation_limits(ctx) {
        errors.push(err);
    }
    if let Err(err) = ensure_introspection_is_accepted(ctx) {
        errors.push(err);
    }

    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

fn ensure_introspection_is_accepted(ctx: OperationContext<'_>) -> Result<(), ValidationError> {
    if ctx.operation.attributes.ty.is_query() && ctx.schema.settings.disable_introspection {
        for field in ctx.root_selection_set().fields() {
            if let Some(field) = field.as_data() {
                if ctx
                    .schema
                    .subgraphs
                    .introspection
                    .meta_fields
                    .contains(&field.definition_id)
                {
                    return Err(ValidationError::IntrospectionIsDisabled {
                        location: field.location,
                    });
                }
            }
        }
    }

    Ok(())
}

fn enforce_operation_limits(ctx: OperationContext<'_>) -> Result<(), ValidationError> {
    let operation_limits = &ctx.schema.settings.operation_limits;
    let selection_set = ctx.root_selection_set();

    // All other limits are verified before the binding step.
    if let Some(max_height) = operation_limits.height {
        let height = selection_set.height(&mut Default::default());
        if height > max_height {
            return Err(ValidationError::QueryTooHigh);
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
