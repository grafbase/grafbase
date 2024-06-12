use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use engine::BatchRequest;
use engine_v2::HttpGraphqlResponse;
use futures::stream::BoxStream;
use graphql_composition::FederatedGraph;
use runtime::fetch::{FetchError, FetchRequest, FetchResponse, FetchResult, GraphqlRequest};

use crate::federation::GraphqlResponse;

#[derive(Clone)]
pub struct FederationGatewayWithoutIO<'a> {
    engine: Arc<engine_v2::Engine>,
    query: &'a str,
    dummy_responses_index: Arc<AtomicUsize>,
}

impl<'a> FederationGatewayWithoutIO<'a> {
    pub fn new<T: serde::Serialize, I>(schema: &str, query: &'a str, subgraphs_responses: I) -> Self
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
            engine_v2::VersionedConfig::V4(engine_v2::config::Config::from_graph(federated_graph)).into_latest();

        let cache = runtime_local::InMemoryCache::runtime(runtime::cache::GlobalCacheConfig {
            enabled: true,
            ..Default::default()
        });

        let engine = engine_v2::Engine::new(
            Arc::new(config.try_into().unwrap()),
            engine_v2::EngineEnv {
                fetcher,
                cache: cache.clone(),
                trusted_documents: runtime::trusted_documents_client::Client::new(
                    runtime_noop::trusted_documents::NoopTrustedDocuments,
                ),
                kv: runtime_local::InMemoryKvStore::runtime(),
                meter: grafbase_tracing::metrics::meter_from_global_provider(),
            },
        );
        Self {
            engine: Arc::new(engine),
            query,
            dummy_responses_index,
        }
    }

    pub async fn raw_execute(&self) -> HttpGraphqlResponse {
        self.dummy_responses_index.store(0, Ordering::Relaxed);
        self.engine
            .execute(
                http::HeaderMap::new(),
                BatchRequest::Single(engine::Request::new(self.query)),
            )
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
