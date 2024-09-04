mod builder;
mod request;
mod subgraph;

use std::sync::Arc;

pub use builder::*;
use bytes::Bytes;
use graphql_mocks::{MockGraphQlServer, ReceivedRequest};
use http_body_util::BodyExt;
pub use request::*;
use runtime_local::hooks::ChannelLogReceiver;
use tower::ServiceExt;
use url::Url;

use crate::engine_v1::GraphQlRequest;

pub struct TestGateway {
    router: axum::Router,
    #[allow(unused)]
    engine: Arc<engine_v2::Engine<TestRuntime>>,
    #[allow(unused)]
    context: TestRuntimeContext,
    subgraphs: subgraph::Subgraphs,
}

pub struct TestRuntimeContext {
    pub access_log_receiver: ChannelLogReceiver,
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

impl TestGateway {
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
            router: self.router.clone(),
            parts,
            body: request.into(),
        }
    }

    pub async fn raw_execute(&self, request: http::Request<impl Into<axum::body::Body>>) -> http::Response<Bytes> {
        let (parts, body) = request.into_parts();
        let (parts, body) = self
            .router
            .clone()
            .oneshot(http::Request::from_parts(parts, body.into()))
            .await
            .unwrap()
            .into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        http::Response::from_parts(parts, bytes)
    }

    pub fn subgraph<S: graphql_mocks::Subgraph>(&self) -> &MockSubgraph {
        self.subgraphs.get_mock_by_type::<S>().unwrap()
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
