use crate::{response::GraphqlError, ErrorCode};

#[derive(thiserror::Error, Debug)]
pub(crate) enum PlanError {
    #[error("Internal Error")]
    InternalError,
    #[error(transparent)]
    QueryPlanning(#[from] query_solver::Error),
}

impl From<PlanError> for GraphqlError {
    fn from(err: PlanError) -> Self {
        GraphqlError::new(err.to_string(), ErrorCode::OperationPlanningError)
    }
}
