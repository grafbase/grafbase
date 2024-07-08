use std::borrow::Cow;

use crate::response::{ErrorCode, GraphqlError};

#[derive(thiserror::Error, Debug)]
pub enum ExecutionError {
    #[error("Internal error: {0}")]
    Internal(Cow<'static, str>),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error(transparent)]
    Fetch(#[from] runtime::fetch::FetchError),
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
        }
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
