use crate::response::{ErrorCode, GraphqlError};

pub(crate) mod collect;
mod conditions;
mod walker_ext;

pub type PlanningResult<T> = Result<T, PlanningError>;

#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<PlanningError> for GraphqlError {
    fn from(error: PlanningError) -> Self {
        let message = error.to_string();
        GraphqlError::new(message, ErrorCode::OperationPlanningError)
    }
}

impl From<String> for PlanningError {
    fn from(error: String) -> Self {
        PlanningError::InternalError(error)
    }
}

impl From<&str> for PlanningError {
    fn from(error: &str) -> Self {
        PlanningError::InternalError(error.to_string())
    }
}
