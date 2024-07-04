use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use engine::BatchRequest;
use engine_v2::HttpGraphqlResponse;
use futures::{stream::BoxStream, StreamExt, TryStreamExt};
use gateway_core::StreamingFormat;
use graphql_composition::FederatedGraph;
use headers::HeaderMapExt;
use runtime::{
    fetch::{FetchError, FetchRequest, FetchResponse, FetchResult, GraphqlRequest},
    hooks::DynamicHooks,
};

use crate::federation::{GraphqlResponse, GraphqlStreamingResponse};

use super::TestRuntime;

#[derive(Clone)]
pub struct DeterministicEngine<'a> {
    engine: Arc<engine_v2::Engine<TestRuntime>>,
    query: &'a str,
    dummy_responses_index: Arc<AtomicUsize>,
}

pub struct DeterministicEngineBuilder<'a> {
    hooks: DynamicHooks,
    schema: &'a str,
    query: &'a str,
    subgraphs_json_responses: Vec<String>,
}

impl<'a> DeterministicEngineBuilder<'a> {
    #[must_use]
    pub fn with_subgraph_response<T: serde::Serialize>(mut self, resp: T) -> Self {
        self.subgraphs_json_responses
            .push(serde_json::to_string(&resp).unwrap());
        self
    }

    #[must_use]
    pub fn with_hooks(mut self, hooks: impl Into<DynamicHooks>) -> Self {
        self.hooks = hooks.into();
        self
    }

    pub async fn build(self) -> DeterministicEngine<'a> {
        let dummy_responses_index = Arc::new(AtomicUsize::new(0));
        let fetcher = DummyFetcher::create(
            self.subgraphs_json_responses
                .into_iter()
                .map(|resp| FetchResponse {
                    bytes: resp.into_bytes().into(),
                })
                .collect(),
            dummy_responses_index.clone(),
        );
        let federated_graph = FederatedGraph::from_sdl(self.schema).unwrap().into_latest();
        let config =
            engine_v2::VersionedConfig::V4(engine_v2::config::Config::from_graph(federated_graph)).into_latest();

        let engine = engine_v2::Engine::new(
            Arc::new(config.try_into().unwrap()),
            None,
            TestRuntime {
                fetcher,
                trusted_documents: runtime::trusted_documents_client::Client::new(
                    runtime_noop::trusted_documents::NoopTrustedDocuments,
                ),
                kv: runtime_local::InMemoryKvStore::runtime(),
                meter: grafbase_tracing::metrics::meter_from_global_provider(),
                hooks: self.hooks,
            },
        )
        .await;
        DeterministicEngine {
            engine: Arc::new(engine),
            query: self.query,
            dummy_responses_index,
        }
    }
}

impl<'a> DeterministicEngine<'a> {
    pub fn builder(schema: &'a str, query: &'a str) -> DeterministicEngineBuilder<'a> {
        DeterministicEngineBuilder {
            hooks: DynamicHooks::default(),
            schema,
            query,
            subgraphs_json_responses: Vec::new(),
        }
    }

    pub async fn new<T: serde::Serialize, I>(schema: &'a str, query: &'a str, subgraphs_responses: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut builder = Self::builder(schema, query);
        for resp in subgraphs_responses {
            builder = builder.with_subgraph_response(resp);
        }
        builder.build().await
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

    pub async fn execute_stream(&self) -> GraphqlStreamingResponse {
        self.dummy_responses_index.store(0, Ordering::Relaxed);
        let mut headers = http::HeaderMap::new();
        headers.typed_insert(StreamingFormat::IncrementalDelivery);
        let response = self
            .engine
            .execute(headers, BatchRequest::Single(engine::Request::new(self.query)))
            .await;
        let stream = multipart_stream::parse(response.body.into_stream().map_ok(Into::into), "-")
            .map(|result| serde_json::from_slice(&result.unwrap().body).unwrap());
        GraphqlStreamingResponse {
            stream: Box::pin(stream),
            headers: response.headers,
        }
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
