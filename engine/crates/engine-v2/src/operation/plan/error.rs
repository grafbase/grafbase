use crate::{response::GraphqlError, ErrorCode};

pub(crate) type PlanResult<T> = std::result::Result<T, PlanError>;

#[derive(thiserror::Error, Debug)]
pub(crate) enum PlanError {
    #[error("Internal Error")]
    InternalError,
}

impl From<PlanError> for GraphqlError {
    fn from(err: PlanError) -> Self {
        GraphqlError::new(err.to_string(), ErrorCode::OperationPlanningError)
    }
}
