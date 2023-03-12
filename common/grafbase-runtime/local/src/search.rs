use grafbase_runtime::{
    search::{Request, Response, SearchEngine, SearchEngineInner, SearchError},
    ExecutionContext,
};
use search_protocol::{QueryExecutionRequest, QueryExecutionResponse};

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
    async fn search(&self, ctx: &ExecutionContext, request: Request) -> Result<Response, SearchError> {
        self.bridge
            .request::<QueryExecutionRequest, QueryExecutionResponse>(
                "/search",
                TryFrom::try_from(request).map_err(|_| SearchError::ServerError)?,
            )
            .await
            .map_err(|error| {
                log::error!(ctx.request_id, "Search Request failed with: {}", error);
                SearchError::ServerError
            })
    }
}
