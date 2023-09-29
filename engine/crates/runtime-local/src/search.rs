use graph_entities::NodeID;
use runtime::search::{QueryError, Request, Response, SearchEngine, SearchEngineInner};
use search_protocol::query::{Query, QueryRequest, QueryResponse, QueryResponseDiscriminants, QueryResponseParameters};

use crate::bridge::Bridge;

pub struct LocalSearchEngine {
    bridge: Bridge,
}

impl LocalSearchEngine {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(bridge: Bridge) -> SearchEngine {
        SearchEngine::new(Box::new(LocalSearchEngine { bridge }))
    }
}

#[async_trait::async_trait]
impl SearchEngineInner for LocalSearchEngine {
    async fn query(&self, ctx: &runtime::Context, request: Request) -> Response {
        let Request {
            query,
            pagination,
            index,
        } = request;
        let request = QueryRequest {
            query: Query::try_from(query)?,
            pagination,
            index: index.clone(),
            database: String::new(),
            ray_id: ctx.ray_id().to_string(),
            response_parameters: QueryResponseParameters {
                version: QueryResponseDiscriminants::V1,
            },
        };
        self.bridge
            .request::<QueryRequest, QueryResponse>("search", request)
            .await
            .map_err(|error| {
                log::error!(ctx.ray_id(), "Search Request failed with: {}", error);
                QueryError::ServerError
            })?
            .normalized(|ulid| NodeID::new(&index, &ulid.to_string()).to_string())
    }
}
