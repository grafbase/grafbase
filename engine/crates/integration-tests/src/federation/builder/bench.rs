use std::{
    borrow::Cow,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use bytes::Bytes;
use engine::BatchRequest;
use engine_v2::HttpGraphqlResponse;
use futures::{StreamExt, TryStreamExt};
use gateway_core::StreamingFormat;
use graphql_composition::FederatedGraph;
use headers::HeaderMapExt;
use runtime::{
    bytes::OwnedOrSharedBytes,
    fetch::{dynamic::DynFetcher, FetchRequest, FetchResult},
    hooks::DynamicHooks,
};
use runtime_local::InMemoryOperationCacheFactory;

use crate::federation::{GraphqlResponse, GraphqlStreamingResponse};

use super::TestRuntime;

#[derive(Clone)]
pub struct DeterministicEngine {
    engine: Arc<engine_v2::Engine<TestRuntime>>,
    query: String,
    dummy_responses_index: Arc<AtomicUsize>,
}

pub struct DeterministicEngineBuilder<'a> {
    runtime: TestRuntime,
    schema: &'a str,
    query: String,
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
        self.runtime.hooks = hooks.into();
        self
    }

    pub fn without_hot_cache(mut self) -> Self {
        self.runtime.hot_cache_factory = InMemoryOperationCacheFactory::inactive();
        self
    }

    pub async fn build(self) -> DeterministicEngine {
        let dummy_responses_index = Arc::new(AtomicUsize::new(0));
        let fetcher = DummyFetcher::new(
            self.subgraphs_json_responses
                .into_iter()
                .map(|resp| http::Response::builder().body(resp.into_bytes().into()).unwrap())
                .collect(),
            dummy_responses_index.clone(),
        );
        let federated_graph = FederatedGraph::from_sdl(self.schema).unwrap().into_latest();
        let config =
            engine_v2::VersionedConfig::V5(engine_v2::config::Config::from_graph(federated_graph)).into_latest();

        let engine = engine_v2::Engine::new(
            Arc::new(config.try_into().unwrap()),
            None,
            TestRuntime {
                fetcher: fetcher.into(),
                ..self.runtime
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

impl DeterministicEngine {
    pub fn builder(schema: &str, query: impl Into<Cow<'static, str>>) -> DeterministicEngineBuilder<'_> {
        let query: Cow<'static, str> = query.into();
        DeterministicEngineBuilder {
            runtime: TestRuntime::default(),
            schema,
            query: query.into_owned(),
            subgraphs_json_responses: Vec::new(),
        }
    }

    pub async fn new<T: serde::Serialize, I>(
        schema: &str,
        query: impl Into<Cow<'static, str>>,
        subgraphs_responses: I,
    ) -> Self
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
                BatchRequest::Single(engine::Request::new(&self.query)),
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
            .execute(headers, BatchRequest::Single(engine::Request::new(&self.query)))
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
    responses: Arc<Vec<http::Response<OwnedOrSharedBytes>>>,
    index: Arc<AtomicUsize>,
}

impl DummyFetcher {
    fn new(responses: Vec<http::Response<OwnedOrSharedBytes>>, index: Arc<AtomicUsize>) -> Self {
        Self {
            responses: Arc::new(responses),
            index,
        }
    }
}

#[async_trait::async_trait]
impl DynFetcher for DummyFetcher {
    async fn fetch(&self, _request: FetchRequest<'_, Bytes>) -> FetchResult<http::Response<OwnedOrSharedBytes>> {
        Ok(self
            .responses
            .get(self.index.fetch_add(1, Ordering::Relaxed))
            .cloned()
            .expect("No more responses"))
    }
}
