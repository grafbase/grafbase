mod builder;
mod request;
mod runtime;
mod subgraph;
mod websocket_request;

use std::sync::Arc;

pub use self::subgraph::Subgraphs;
pub use builder::*;
use bytes::Bytes;
use graphql_mocks::{MockGraphQlServer, ReceivedRequest};
use http_body_util::BodyExt;
pub use request::*;
pub use runtime::*;
use tower::ServiceExt;
use url::Url;
use websocket_request::WebsocketRequest;

#[derive(Clone)]
pub struct Gateway {
    #[allow(unused)]
    tmpdir: Arc<tempfile::TempDir>,
    router: axum::Router,
    #[allow(unused)]
    engine: Arc<engine::Engine<TestRuntime>>,
    subgraphs: subgraph::Subgraphs,
}

impl std::fmt::Debug for Gateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gateway").finish_non_exhaustive()
    }
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
    Gqlgen,
    Mtls,
}

impl DockerSubgraph {
    pub fn name(&self) -> &str {
        match self {
            DockerSubgraph::Sse => "sse",
            DockerSubgraph::Gqlgen => "gqlgen",
            DockerSubgraph::Mtls => "mtls-test-subgraph",
        }
    }

    pub fn url(&self) -> Url {
        match self {
            DockerSubgraph::Sse => Url::parse("http://localhost:4092/graphql").unwrap(),
            DockerSubgraph::Gqlgen => Url::parse("http://localhost:8080/query").unwrap(),
            DockerSubgraph::Mtls => Url::parse("https://localhost:8081/graphql").unwrap(),
        }
    }

    pub fn schema(&self) -> Option<String> {
        let DockerSubgraph::Mtls = self else {
            return None;
        };

        let schema = include_str!("../../data/mtls-subgraph/subgraph.graphql");

        Some(schema.to_string())
    }
}

impl Gateway {
    pub fn builder() -> GatewayBuilder {
        GatewayBuilder::default()
    }

    pub fn get(&self, request: impl Into<GraphQlRequest>) -> TestRequest {
        self.execute(http::Method::GET, "/graphql", request)
    }

    pub fn post(&self, request: impl Into<GraphQlRequest>) -> TestRequest {
        self.execute(http::Method::POST, "/graphql", request)
    }

    pub async fn mcp_sse(&self, path: &str) -> McpStream {
        McpStream::new(self.router.clone(), path).await
    }

    pub fn mcp_http(&self, path: &str) -> McpHttpClientBuilder {
        McpHttpClientBuilder::new(self.router.clone(), path)
    }

    pub fn execute(&self, method: http::Method, path: &str, request: impl Into<GraphQlRequest>) -> TestRequest {
        let (mut parts, _) = http::Request::new(()).into_parts();
        parts.method = method;
        parts.uri = format!("http://127.0.0.1{path}").parse().unwrap();

        TestRequest {
            router: self.router.clone(),
            parts,
            body: request.into(),
        }
    }

    pub fn ws(&self, request: impl Into<GraphQlRequest>) -> WebsocketRequest {
        websocket_request::WebsocketRequest {
            router: self.router.clone(),
            gql: request.into(),
            headers: http::HeaderMap::default(),
            init_payload: None,
            path: "/ws",
        }
    }

    pub async fn raw_execute(&self, request: http::Request<impl Into<axum::body::Body>>) -> http::Response<Bytes> {
        let (parts, body) = request.into_parts();

        let result = self
            .router
            .clone()
            .oneshot(http::Request::from_parts(parts, body.into()))
            .await
            .unwrap();

        let (parts, body) = result.into_parts();
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

    pub fn drain_graphql_requests_sent_to_by_name(&self, name: &str) -> Vec<async_graphql::Request> {
        self.subgraphs
            .get_mock_by_name(name)
            .unwrap()
            .drain_received_requests()
            .map(|req| req.body)
            .collect()
    }
}
