mod introspection;
mod operation_limits;

use crate::{
    execution::ExecutionContext,
    operation::{Location, OperationWalker},
    response::GraphqlError,
};
use introspection::*;
use operation_limits::*;

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
        GraphqlError {
            message: err.to_string(),
            locations,
            ..Default::default()
        }
    }
}

pub(super) fn validate_operation(
    ctx: ExecutionContext<'_>,
    operation: OperationWalker<'_>,
    request: &engine::Request,
) -> Result<(), ValidationError> {
    enforce_operation_limits(&ctx.engine.schema, operation, request)?;
    ensure_introspection_is_accepted(&ctx.engine.schema, operation, request)?;

    Ok(())
}
