use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use worker::{self, Env};

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
    type Fetcher;
    type HealthRequest;
    type HealthResponse;
    type ExecutionRequest;
    type ExecutionResponse;

    fn from_env(env: &Env) -> worker::Result<HashMap<String, String>>;

    async fn execute(
        fetch: Arc<Option<Self::Fetcher>>,
        env: Arc<HashMap<String, String>>,
        execution_request: Self::ExecutionRequest,
    ) -> ExecutionResult<Self::ExecutionResponse>;

    async fn health(
        fetch: Arc<Option<Self::Fetcher>>,
        health_request: Self::HealthRequest,
    ) -> ExecutionResult<Self::HealthResponse>;
}
