mod builder;
mod request;

use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub use builder::*;
use bytes::Bytes;
use engine_v2::Body;
use graphql_mocks::{MockGraphQlServer, ReceivedRequest};
pub use request::*;
use runtime_local::hooks::ChannelLogReceiver;
use url::Url;

use crate::engine_v1::GraphQlRequest;

pub struct TestEngineV2 {
    engine: Arc<engine_v2::Engine<TestRuntime>>,
    mock_subgraphs: HashMap<TypeId, MockSubgraph>,
    #[allow(unused)]
    docker_subgraphs: HashSet<DockerSubgraph>,
    #[allow(unused)]
    access_log_receiver: ChannelLogReceiver,
}

pub struct MockSubgraph {
    pub name: String,
    pub server: MockGraphQlServer,
}

impl std::ops::Deref for MockSubgraph {
    type Target = MockGraphQlServer;
    fn deref(&self) -> &Self::Target {
        &self.server
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum DockerSubgraph {
    Sse,
}

impl DockerSubgraph {
    pub fn name(&self) -> &str {
        match self {
            DockerSubgraph::Sse => "sse",
        }
    }

    pub fn url(&self) -> Url {
        match self {
            DockerSubgraph::Sse => Url::parse("http://localhost:4092/graphql").unwrap(),
        }
    }
}

impl TestEngineV2 {
    pub fn get(&self, request: impl Into<GraphQlRequest>) -> TestRequest {
        self.execute(http::Method::GET, request)
    }

    pub fn post(&self, request: impl Into<GraphQlRequest>) -> TestRequest {
        self.execute(http::Method::POST, request)
    }

    pub fn execute(&self, method: http::Method, request: impl Into<GraphQlRequest>) -> TestRequest {
        let (mut parts, _) = http::Request::new(()).into_parts();
        parts.method = method;
        parts.uri = http::Uri::from_static("http://127.0.0.1/graphql");

        TestRequest {
            engine: Arc::clone(&self.engine),
            parts,
            body: request.into(),
        }
    }

    pub async fn raw_execute(&self, request: http::Request<impl Into<Bytes>>) -> http::Response<Body> {
        let (parts, body) = request.into_parts();
        let body: Bytes = body.into();

        self.engine
            .execute(http::Request::from_parts(parts, Box::pin(async move { Ok(body) })))
            .await
    }

    pub fn subgraph<S: graphql_mocks::Subgraph>(&self) -> &MockSubgraph {
        self.mock_subgraphs.get(&std::any::TypeId::of::<S>()).unwrap()
    }

    pub fn drain_http_requests_sent_to<S: graphql_mocks::Subgraph>(&self) -> Vec<ReceivedRequest> {
        self.subgraph::<S>().drain_received_requests().collect()
    }

    pub fn drain_graphql_requests_sent_to<S: graphql_mocks::Subgraph>(&self) -> Vec<async_graphql::Request> {
        self.subgraph::<S>()
            .drain_received_requests()
            .map(|req| req.body)
            .collect()
    }
}
