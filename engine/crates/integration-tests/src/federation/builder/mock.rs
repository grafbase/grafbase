use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

use futures::stream::BoxStream;
use graphql_composition::FederatedGraph;
use runtime::{
    fetch::{FetchError, FetchRequest, FetchResponse, FetchResult, GraphqlRequest},
    hooks::Hooks,
};
use runtime_noop::hooks::HooksNoop;
use tokio::sync::mpsc;

use crate::{engine_v1::GraphQlRequest, federation::ExecutionRequest};

pub struct MockFederationEngine {
    engine: Arc<engine_v2::Engine>,
    receiver: mpsc::UnboundedReceiver<String>,
    responses: Arc<Mutex<HashMap<String, VecDeque<Vec<u8>>>>>,
}

impl MockFederationEngine {
    pub fn new(schema: &str) -> Self {
        let federated_graph = FederatedGraph::from_sdl(schema).unwrap().into_latest();
        let config =
            engine_v2::VersionedConfig::V4(engine_v2::config::Config::from_graph(federated_graph)).into_latest();

        let cache = runtime_local::InMemoryCache::runtime(runtime::cache::GlobalCacheConfig {
            enabled: true,
            ..Default::default()
        });
        let (sender, receiver) = mpsc::unbounded_channel();
        let responses = Arc::new(Mutex::new(HashMap::new()));
        let fetcher = FetcherMock {
            requests: sender,
            responses: responses.clone(),
        };

        let engine = engine_v2::Engine::new(
            Arc::new(config.try_into().unwrap()),
            engine_v2::EngineEnv {
                fetcher: runtime::fetch::Fetcher::new(fetcher),
                cache: cache.clone(),
                trusted_documents: runtime::trusted_documents_client::Client::new(
                    runtime_noop::trusted_documents::NoopTrustedDocuments,
                ),
                kv: runtime_local::InMemoryKvStore::runtime(),
                meter: grafbase_tracing::metrics::meter_from_global_provider(),
                user_hooks: Hooks::new(HooksNoop),
            },
        );
        Self {
            engine: Arc::new(engine),
            receiver,
            responses,
        }
    }

    pub fn mock<R: serde::Serialize>(&mut self, host: &str, responses: impl IntoIterator<Item = R>) {
        let responses = responses.into_iter().map(|r| serde_json::to_vec(&r).unwrap()).collect();
        self.responses.lock().unwrap().insert(host.to_string(), responses);
    }

    pub fn received_requests(&mut self) -> Vec<String> {
        std::iter::from_fn(|| self.receiver.try_recv().ok()).collect()
    }

    pub fn execute(&self, request: impl Into<GraphQlRequest>) -> ExecutionRequest {
        ExecutionRequest {
            request: request.into(),
            headers: HashMap::new(),
            engine: Arc::clone(&self.engine),
        }
    }
}

struct FetcherMock {
    requests: mpsc::UnboundedSender<String>,
    responses: Arc<Mutex<HashMap<String, VecDeque<Vec<u8>>>>>,
}

#[async_trait::async_trait]
impl runtime::fetch::FetcherInner for FetcherMock {
    async fn post(&self, request: FetchRequest<'_>) -> FetchResult<FetchResponse> {
        self.requests.send(request.json_body).unwrap();
        self.responses
            .lock()
            .unwrap()
            .get_mut(request.url.host_str().unwrap())
            .and_then(|responses| responses.pop_front())
            .map(|bytes| FetchResponse { bytes: bytes.into() })
            .ok_or(FetchError::any("No more responses"))
    }

    async fn stream(
        &self,
        _request: GraphqlRequest<'_>,
    ) -> FetchResult<BoxStream<'static, Result<serde_json::Value, FetchError>>> {
        unreachable!()
    }
}
