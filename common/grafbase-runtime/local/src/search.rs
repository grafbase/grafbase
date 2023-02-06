use grafbase_runtime::{
    search::{SearchEngine, SearchEngineInner, SearchError, SearchRequest, SearchResponse},
    ExecutionContext,
};

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
    async fn search(&self, ctx: &ExecutionContext, request: SearchRequest) -> Result<SearchResponse, SearchError> {
        self.bridge
            .request::<SearchRequest, SearchResponse>("/search", request)
            .await
            .map_err(|error| {
                log::error!(ctx.request_id, "Search Request failed with: {}", error);
                SearchError::ServerError
            })
    }
}
