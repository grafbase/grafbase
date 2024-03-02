use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use engine::{HttpGraphqlRequest, HttpGraphqlResponse};
use engine_v2::Engine;
use futures::stream::BoxStream;
use graphql_composition::FederatedGraph;
use runtime::fetch::{FetchError, FetchRequest, FetchResponse, FetchResult, GraphqlRequest};

use crate::federation::GraphqlResponse;

#[derive(Clone)]
pub struct FederationGatewayWithoutIO {
    engine: Arc<Engine>,
    request: Arc<HttpGraphqlRequest<'static>>,
    dummy_responses_index: Arc<AtomicUsize>,
}

impl FederationGatewayWithoutIO {
    pub fn new<T: serde::Serialize, I>(schema: &str, query: &str, subgraphs_responses: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let dummy_responses_index = Arc::new(AtomicUsize::new(0));
        let fetcher = DummyFetcher::create(
            subgraphs_responses
                .into_iter()
                .map(|resp| FetchResponse {
                    bytes: serde_json::to_vec(&resp).unwrap().into(),
                })
                .collect(),
            dummy_responses_index.clone(),
        );
        let federated_graph = FederatedGraph::from_sdl(schema).unwrap().into_latest();
        let config =
            engine_v2::VersionedConfig::V3(engine_v2::config::Config::from_graph(federated_graph)).into_latest();
        let async_runtime = runtime_local::TokioCurrentRuntime::runtime();
        let cache = runtime_local::InMemoryCache::runtime(async_runtime.clone());

        let engine = Engine::new(
            config.into(),
            ulid::Ulid::new().to_string().into(),
            engine_v2::EngineEnv {
                fetcher,
                cache,
                cache_opeartion_cache_control: false,
                trusted_documents: runtime_noop::trusted_documents::NoopTrustedDocuments.into(),
                async_runtime,
                kv: runtime_local::InMemoryKvStore::runtime(),
            },
        );

        Self {
            engine: Arc::new(engine),
            request: Arc::new(HttpGraphqlRequest::JsonBody(
                serde_json::to_vec(&serde_json::json!({
                    "query": query
                }))
                .unwrap()
                .into(),
            )),
            dummy_responses_index,
        }
    }

    pub async fn raw_execute(&self) -> HttpGraphqlResponse {
        self.dummy_responses_index.store(0, Ordering::Relaxed);
        self.engine
            .execute(http::HeaderMap::new(), "", self.request.as_ref().as_ref())
            .await
    }

    pub async fn execute(&self) -> GraphqlResponse {
        self.raw_execute().await.try_into().unwrap()
    }
}

struct DummyFetcher {
    responses: Arc<Vec<FetchResponse>>,
    index: Arc<AtomicUsize>,
}

impl DummyFetcher {
    fn create(responses: Vec<FetchResponse>, index: Arc<AtomicUsize>) -> runtime::fetch::Fetcher {
        runtime::fetch::Fetcher::new(Self {
            responses: Arc::new(responses),
            index,
        })
    }
}

#[async_trait::async_trait]
impl runtime::fetch::FetcherInner for DummyFetcher {
    async fn post(&self, _request: FetchRequest<'_>) -> FetchResult<FetchResponse> {
        Ok(self
            .responses
            .get(self.index.fetch_add(1, Ordering::Relaxed))
            .cloned()
            .expect("No more responses"))
    }

    async fn stream(
        &self,
        _request: GraphqlRequest<'_>,
    ) -> FetchResult<BoxStream<'static, Result<serde_json::Value, FetchError>>> {
        unreachable!()
    }
}
