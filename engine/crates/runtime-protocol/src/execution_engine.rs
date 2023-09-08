use std::sync::Arc;

use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum ExecutionError {
    #[error("Remote execution error: {0}")]
    Remote(#[from] worker::Error),
    #[error("Internal Error: {0}")]
    InternalError(String),
}

pub type ExecutionResult<T> = Result<T, ExecutionError>;

/// Owned trait with 'static in mind
#[async_trait(?Send)]
pub trait ExecutionEngine {
    type ConfigType;
    type ExecutionResponse; // This is always engine::Response (but is needed for tests)

    async fn execute(
        self: Arc<Self>,
        execution_request: crate::ExecutionRequest<Self::ConfigType>,
    ) -> ExecutionResult<Self::ExecutionResponse>;

    async fn health(
        self: Arc<Self>,
        health_request: crate::ExecutionHealthRequest<Self::ConfigType>,
    ) -> ExecutionResult<crate::ExecutionHealthResponse>;
}
