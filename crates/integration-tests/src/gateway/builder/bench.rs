use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use bytes::Bytes;
use engine::{Body, RequestExtensions};
use event_queue::SubgraphResponseBuilder;
use futures::{StreamExt, TryStreamExt};
use runtime::{
    authentication::LegacyToken,
    fetch::{FetchRequest, FetchResult, dynamic::DynFetcher},
};
use runtime_local::InMemoryOperationCache;

use crate::gateway::{ExtContext, GraphqlResponse, GraphqlStreamingResponse};

use super::TestRuntime;

#[derive(Clone)]
pub struct DeterministicEngine {
    engine: Arc<engine::ContractAwareEngine<TestRuntime>>,
    request_parts: http::request::Parts,
    body: Bytes,
    dummy_responses_index: Arc<AtomicUsize>,
}

pub struct DeterministicEngineBuilder<'a> {
    runtime: TestRuntime,
    schema: &'a str,
    query: &'a str,
    subgraphs_json_responses: Vec<String>,
}

impl DeterministicEngineBuilder<'_> {
    #[must_use]
    pub fn with_subgraph_response<T: serde::Serialize>(mut self, resp: T) -> Self {
        self.subgraphs_json_responses
            .push(serde_json::to_string(&resp).unwrap());
        self
    }

    #[must_use]
    pub fn without_operation_cache(mut self) -> Self {
        self.runtime.operation_cache = InMemoryOperationCache::inactive();
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

        let schema = engine::Schema::from_sdl_or_panic(self.schema).await;
        let engine = engine::ContractAwareEngine::new(
            Arc::new(schema),
            TestRuntime {
                fetcher: fetcher.into(),
                ..self.runtime
            },
        );
        let body = Bytes::from(serde_json::to_vec(&serde_json::json!({"query": self.query})).unwrap());
        DeterministicEngine {
            engine: Arc::new(engine),
            request_parts: http::Request::builder()
                .method(http::Method::POST)
                .header(http::header::ACCEPT, http::HeaderValue::from_static("application/json"))
                .header(
                    http::header::CONTENT_TYPE,
                    http::HeaderValue::from_static("application/json"),
                )
                .extension(RequestExtensions {
                    context: ExtContext::default(),
                    token: LegacyToken::Anonymous,
                    contract_key: None,
                })
                .body(())
                .unwrap()
                .into_parts()
                .0,
            body,
            dummy_responses_index,
        }
    }
}

impl DeterministicEngine {
    pub fn builder<'a>(schema: &'a str, query: &'a str) -> DeterministicEngineBuilder<'a> {
        DeterministicEngineBuilder {
            runtime: Default::default(),
            schema,
            query,
            subgraphs_json_responses: Vec::new(),
        }
    }

    pub async fn new<T: serde::Serialize, I>(schema: &str, query: &str, subgraphs_responses: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut builder = Self::builder(schema, query);
        for resp in subgraphs_responses {
            builder = builder.with_subgraph_response(resp);
        }
        builder.build().await
    }

    pub async fn raw_execute(&self) -> http::Response<Body> {
        self.dummy_responses_index.store(0, Ordering::Relaxed);
        self.engine
            .execute(http::Request::from_parts(
                self.request_parts.clone(),
                Box::pin(async { Ok(self.body.clone()) }),
            ))
            .await
    }

    pub async fn execute(&self) -> GraphqlResponse {
        let (parts, body) = self.raw_execute().await.into_parts();
        let bytes = body.into_bytes().unwrap();
        http::Response::from_parts(parts, bytes).try_into().unwrap()
    }

    pub async fn execute_stream(&self) -> GraphqlStreamingResponse {
        self.dummy_responses_index.store(0, Ordering::Relaxed);
        let request = {
            let mut parts = self.request_parts.clone();
            parts.headers.insert(
                http::header::ACCEPT,
                http::HeaderValue::from_static("multipart/mixed,application/json;q=0.9"),
            );
            http::Request::from_parts(parts, Box::pin(async { Ok(self.body.clone()) }))
        };
        let (parts, body) = self.engine.execute(request).await.into_parts();
        let stream = multipart_stream::parse(body.into_stream().map_ok(Into::into), "-")
            .map(|result| serde_json::from_slice(&result.unwrap().body).unwrap())
            .boxed();
        GraphqlStreamingResponse {
            status: parts.status,
            headers: parts.headers,
            stream,
        }
    }
}

struct DummyFetcher {
    responses: Arc<Vec<http::Response<Bytes>>>,
    index: Arc<AtomicUsize>,
}

impl DummyFetcher {
    fn new(responses: Vec<http::Response<Bytes>>, index: Arc<AtomicUsize>) -> Self {
        Self {
            responses: Arc::new(responses),
            index,
        }
    }
}

#[async_trait::async_trait]
impl DynFetcher for DummyFetcher {
    async fn fetch(
        &self,
        _request: FetchRequest<'_, Bytes>,
    ) -> (FetchResult<http::Response<Bytes>>, Option<SubgraphResponseBuilder>) {
        let result = Ok(self
            .responses
            .get(self.index.fetch_add(1, Ordering::Relaxed))
            .cloned()
            .expect("No more responses"));

        (result, None)
    }
}
