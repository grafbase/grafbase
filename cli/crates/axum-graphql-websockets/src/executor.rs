pub trait Executor /* Unpin + Clone + Send + Sync + 'static */ {
    /// Execute a GraphQL query.
    async fn execute(&self, request: Request) -> Response;

    /// Execute a GraphQL subscription with session data.
    fn execute_stream(&self, request: Request, session_data: Option<Arc<Data>>) -> BoxStream<'static, Response>;
}
