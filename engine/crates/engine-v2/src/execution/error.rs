use crate::response::GraphqlError;

#[derive(thiserror::Error, Debug)]
pub enum ExecutionError {
    #[error("Internal error: {0}")]
    Internal(String),
    #[error(transparent)]
    Fetch(#[from] runtime::fetch::FetchError),
}

pub type ExecutionResult<T> = Result<T, ExecutionError>;

impl From<ExecutionError> for GraphqlError {
    fn from(err: ExecutionError) -> Self {
        GraphqlError {
            message: err.to_string(),
            ..Default::default()
        }
    }
}

impl From<&str> for ExecutionError {
    fn from(message: &str) -> Self {
        Self::Internal(message.to_string())
    }
}

impl From<String> for ExecutionError {
    fn from(message: String) -> Self {
        Self::Internal(message)
    }
}
