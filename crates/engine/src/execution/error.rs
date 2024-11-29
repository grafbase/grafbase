use std::borrow::Cow;

use runtime::fetch::FetchError;

use crate::response::{ErrorCode, GraphqlError};

#[derive(thiserror::Error, Debug)]
pub enum ExecutionError {
    #[error("Internal error: {0}")]
    Internal(Cow<'static, str>),
    #[error("Request to subgraph '{subgraph_name}' failed with: {error}")]
    Fetch { subgraph_name: String, error: FetchError },
    #[error(transparent)]
    RateLimit(#[from] runtime::rate_limiting::Error),
    #[error("{0}")]
    Graphql(GraphqlError),
}

impl ExecutionError {
    pub fn as_fetch_invalid_status_code(&self) -> Option<http::StatusCode> {
        match self {
            Self::Fetch {
                error: FetchError::InvalidStatusCode(code),
                ..
            } => Some(*code),
            _ => None,
        }
    }
}

pub type ExecutionResult<T> = Result<T, ExecutionError>;

impl From<ExecutionError> for GraphqlError {
    fn from(err: ExecutionError) -> Self {
        if let ExecutionError::Graphql(err) = err {
            return err;
        }
        let message = err.to_string();
        let code = match &err {
            ExecutionError::Internal(_) => ErrorCode::InternalServerError,
            ExecutionError::Fetch { .. } => ErrorCode::SubgraphRequestError,
            ExecutionError::RateLimit(_) => ErrorCode::RateLimited,
            ExecutionError::Graphql(err) => err.code,
        };
        GraphqlError::new(message, code)
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
