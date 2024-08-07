mod builder;
mod request;

use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub use builder::*;
use graphql_mocks::{MockGraphQlServer, ReceivedRequest};
pub use request::*;
use url::Url;

use crate::engine_v1::GraphQlRequest;

pub struct TestEngineV2 {
    engine: Arc<engine_v2::Engine<TestRuntime>>,
    mock_subgraphs: HashMap<TypeId, MockSubgraph>,
    #[allow(unused)]
    docker_subgraphs: HashSet<DockerSubgraph>,
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
    pub fn execute(&self, request: impl Into<GraphQlRequest>) -> ExecutionRequest {
        ExecutionRequest {
            request: request.into(),
            headers: Vec::new(),
            engine: Arc::clone(&self.engine),
        }
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
