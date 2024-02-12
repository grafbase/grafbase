use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use engine::RequestHeaders;
use futures::stream::BoxStream;
use gateway_v2::Response;
use graphql_composition::FederatedGraph;
use runtime::fetch::{FetchError, FetchRequest, FetchResponse, FetchResult, GraphqlRequest};

use crate::engine::RequestContext;

#[derive(Clone)]
pub struct FederationGatewayWithoutIO<'a> {
    gateway: Arc<gateway_v2::Gateway>,
    query: &'a str,
    ctx: Arc<RequestContext>,
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

        let gateway = gateway_v2::Gateway::new(
            config.into(),
            engine_v2::EngineEnv {
                fetcher,
                cache: cache.clone(),
            },
            gateway_v2::GatewayEnv {
                kv: runtime_local::InMemoryKvStore::runtime(),
                cache,
            },
        );
        let (ctx, _) = RequestContext::new(HashMap::with_capacity(0));
        let ctx = Arc::new(ctx);
        Self {
            gateway: Arc::new(gateway),
            query,
            ctx,
            dummy_responses_index,
        }
    }

    pub async fn execute(&self) -> Response {
        let response = self.unchecked_execute().await;
        assert!(
            response.status.is_success() && !response.has_errors,
            "Execution failed!\n{}",
            String::from_utf8_lossy(&response.bytes)
        );
        response
    }

    pub async fn unchecked_execute(&self) -> Response {
        self.dummy_responses_index.store(0, Ordering::Relaxed);
        let session = self.gateway.authorize(RequestHeaders::default()).await.unwrap();
        session
            .execute(self.ctx.as_ref(), engine::Request::new(self.query))
            .await
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
