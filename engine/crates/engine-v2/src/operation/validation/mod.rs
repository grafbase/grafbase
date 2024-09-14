mod introspection;
mod operation_limits;

use crate::{
    operation::{Location, OperationWalker},
    response::{ErrorCode, GraphqlError},
};
use introspection::*;
use operation_limits::*;
use schema::Schema;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ValidationError {
    #[error(transparent)]
    OperationLimitExceeded(#[from] OperationLimitExceededError),
    #[error("GraphQL introspection is not allowed, but the query contained __schema or __type")]
    IntrospectionWhenDisabled { location: Location },
}

impl From<ValidationError> for GraphqlError {
    fn from(err: ValidationError) -> Self {
        let locations = match &err {
            ValidationError::IntrospectionWhenDisabled { location } => vec![*location],
            ValidationError::OperationLimitExceeded { .. } => Vec::new(),
        };
        GraphqlError::new(err.to_string(), ErrorCode::OperationValidationError).with_locations(locations)
    }
}

/// Validates a GraphQL operation against the provided schema.
///
/// This function checks the operation for compliance with operation limits
/// and ensures that introspection queries are allowed if required.
///
/// # Arguments
///
/// * `schema` - A reference to the schema used for validation.
/// * `operation` - An `OperationWalker` representing the operation to validate.
///
/// # Returns
///
/// Returns `Ok(())` if the operation is valid.
/// If the operation exceeds limits or contains disallowed introspection,
/// it returns a `ValidationError`.
pub(super) fn validate_operation(schema: &Schema, operation: OperationWalker<'_>) -> Result<(), ValidationError> {
    enforce_operation_limits(schema, operation)?;
    ensure_introspection_is_accepted(schema, operation)?;

    Ok(())
}
