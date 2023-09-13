use std::sync::Arc;

use async_trait::async_trait;
use futures_util::future::BoxFuture;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum ExecutionError {
    #[error("Remote execution error: {0}")]
    Remote(#[from] worker::Error),
    #[error("Internal Error: {0}")]
    InternalError(String),
}

pub type ExecutionResult<T> = Result<T, ExecutionError>;

/// The format execute_stream should return
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamingFormat {
    /// Follow the [incremental delivery spec][1]
    ///
    /// [1]: https://github.com/graphql/graphql-over-http/blob/main/rfcs/IncrementalDelivery.md
    IncrementalDelivery,
    /// Follow the [GraphQL over SSE spec][1]
    ///
    /// [1]: https://github.com/graphql/graphql-over-http/blob/main/rfcs/GraphQLOverSSE.md
    GraphQLOverSSE,
}

impl StreamingFormat {
    pub fn from_accept_header(header: &str) -> Option<Self> {
        // Note: This is not even close to the correct way to parse the Accept header.
        // Going to improve apon this in GB-4878
        match header {
            "multipart/mixed" => Some(Self::IncrementalDelivery),
            "text/event-stream" => Some(Self::GraphQLOverSSE),
            _ => None,
        }
    }
}

/// Owned trait with 'static in mind
#[async_trait(?Send)]
pub trait ExecutionEngine {
    type ConfigType;
    type ExecutionResponse; // This is always grafbase_engine::Response (but is needed for tests)

    async fn execute(
        self: Arc<Self>,
        execution_request: crate::ExecutionRequest<Self::ConfigType>,
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
        execution_request: crate::ExecutionRequest<Self::ConfigType>,
        streaming_format: StreamingFormat,
    ) -> ExecutionResult<(worker::Response, Option<BoxFuture<'static, ()>>)>;

    async fn health(
        self: Arc<Self>,
        health_request: crate::ExecutionHealthRequest<Self::ConfigType>,
    ) -> ExecutionResult<crate::ExecutionHealthResponse>;
}
