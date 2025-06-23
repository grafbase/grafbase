use error::ErrorResponse;

use crate::response::GraphqlError;

pub(crate) type PlanResult<T> = std::result::Result<T, PlanError>;

#[derive(Debug)]
pub(crate) enum PlanError {
    Internal,
    GraphqlError(GraphqlError),
    ErrorResponse(Box<ErrorResponse>),
}

impl From<GraphqlError> for PlanError {
    fn from(err: GraphqlError) -> Self {
        PlanError::GraphqlError(err)
    }
}

impl From<ErrorResponse> for PlanError {
    fn from(err: ErrorResponse) -> Self {
        PlanError::ErrorResponse(Box::new(err))
    }
}
