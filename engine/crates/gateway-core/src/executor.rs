use std::sync::Arc;

use common_types::auth::ExecutionAuth;

#[async_trait::async_trait]
pub trait Executor: Send + Sync {
    type Error;
    type Context;

    // The response used for streaming.
    //
    // This is generally some http Response type (e.g. http::Response, axum::Response)
    // that reads from a Stream
    type StreamingResponse;

    // Caching can defer the actual execution of the request when the data is stale for example.
    // To simplify our code, instead of having a 'ctx lifetime, we expect those "background"
    // futures to be 'static. Hence this method requires an Arc<Self>.
    async fn execute(
        self: Arc<Self>,
        ctx: Arc<Self::Context>,
        auth: ExecutionAuth,
        request: engine::Request,
    ) -> Result<engine::Response, Self::Error>;

    async fn execute_stream(
        self: Arc<Self>,
        ctx: Arc<Self::Context>,
        auth: ExecutionAuth,
        request: engine::Request,
        streaming_format: crate::StreamingFormat,
    ) -> Result<Self::StreamingResponse, Self::Error>;
}
