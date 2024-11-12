use crate::{response::GraphqlError, ErrorCode};

pub(crate) type SolveResult<T> = Result<T, SolveError>;

#[derive(thiserror::Error, Debug)]
pub(crate) enum SolveError {
    #[error("Internal Error")]
    InternalError,
    #[error(transparent)]
    QueySolver(#[from] query_solver::Error),
}

impl From<SolveError> for GraphqlError {
    fn from(err: SolveError) -> Self {
        GraphqlError::new(err.to_string(), ErrorCode::OperationPlanningError)
    }
}
