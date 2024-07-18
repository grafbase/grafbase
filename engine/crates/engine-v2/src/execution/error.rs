use std::borrow::Cow;

use crate::response::{ErrorCode, GraphqlError};

pub(crate) type PlanningResult<T> = Result<T, PlanningError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum PlanningError {
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<PlanningError> for GraphqlError {
    fn from(error: PlanningError) -> Self {
        GraphqlError::new(error.to_string(), ErrorCode::OperationPlanningError)
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

#[derive(thiserror::Error, Debug)]
pub enum ExecutionError {
    #[error("Internal error: {0}")]
    Internal(Cow<'static, str>),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error(transparent)]
    Fetch(#[from] runtime::fetch::FetchError),
    #[error(transparent)]
    RateLimit(#[from] runtime::rate_limiting::Error),
    #[error("{0}")]
    Graphql(GraphqlError),
}

pub type ExecutionResult<T> = Result<T, ExecutionError>;

impl From<ExecutionError> for GraphqlError {
    fn from(err: ExecutionError) -> Self {
        match err {
            ExecutionError::Internal(message) => GraphqlError::new(message, ErrorCode::InternalServerError),
            ExecutionError::DeserializationError(message) => {
                GraphqlError::new(message, ErrorCode::SubgraphInvalidResponseError)
            }
            ExecutionError::Fetch(err) => GraphqlError::new(err.to_string(), ErrorCode::SubgraphRequestError),
            ExecutionError::RateLimit(err) => GraphqlError::new(err.to_string(), ErrorCode::RateLimited),
            ExecutionError::Graphql(err) => err,
        }
    }
}

impl From<GraphqlError> for ExecutionError {
    fn from(err: GraphqlError) -> Self {
        ExecutionError::Graphql(err)
    }
}

impl From<&'static str> for ExecutionError {
    fn from(message: &'static str) -> Self {
        Self::Internal(message.into())
    }
}

impl From<String> for ExecutionError {
    fn from(message: String) -> Self {
        Self::Internal(message.into())
    }
}

impl From<serde_json::Error> for ExecutionError {
    fn from(err: serde_json::Error) -> Self {
        Self::DeserializationError(err.to_string())
    }
}
