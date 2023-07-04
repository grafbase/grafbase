use grafbase_runtime::{
    search::{Request, Response, SearchEngine, SearchEngineInner, SearchError},
    GraphqlRequestExecutionContext,
};
use search_query::{QueryExecutionRequest, QueryExecutionResponse};

use crate::bridge::Bridge;

pub struct LocalSearchEngine {
    bridge: Bridge,
}

impl LocalSearchEngine {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(bridge_port: u16) -> SearchEngine {
        SearchEngine::new(Box::new(LocalSearchEngine {
            bridge: Bridge::new(bridge_port),
        }))
    }
}

#[async_trait::async_trait]
impl SearchEngineInner for LocalSearchEngine {
    async fn search(&self, ctx: &GraphqlRequestExecutionContext, request: Request) -> Result<Response, SearchError> {
        self.bridge
            .request::<QueryExecutionRequest, QueryExecutionResponse>(
                "/search",
                QueryExecutionRequest::try_build(request, "", &ctx.ray_id).map_err(|err| {
                    log::error!(ctx.ray_id, "Failed to build QueryExecutionRequest: {err}");
                    SearchError::ServerError
                })?,
            )
            .await
            .map_err(|error| {
                log::error!(ctx.ray_id, "Search Request failed with: {}", error);
                SearchError::ServerError
            })
    }
}
