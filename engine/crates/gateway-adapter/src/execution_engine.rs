use std::sync::Arc;

use async_trait::async_trait;
use futures_util::future::BoxFuture;
use gateway_core::StreamingFormat;

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
    type ExecutionResponse; // This is always grafbase_engine::Response (but is needed for tests)

    async fn execute(
        self: Arc<Self>,
        execution_request: crate::ExecutionRequest,
    ) -> ExecutionResult<Self::ExecutionResponse>;

    /// Executes a streaming request from the engine.
    ///
    /// For streaming requests we current expect to just return the Response from upstream directly
    /// without any caching etc.  At some point we might want to consider caching for streamed requests
    /// but it's not straightforward and requires some thought.
    ///
    /// Note that this returns a Response _and_ an optional future.  That future (if provided) needs to
    /// be polled within a request context.
    async fn execute_stream(
        self: Arc<Self>,
        execution_request: crate::ExecutionRequest,
        streaming_format: StreamingFormat,
    ) -> ExecutionResult<(worker::Response, Option<BoxFuture<'static, ()>>)>;
}
