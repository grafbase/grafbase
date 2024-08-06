mod builder;
mod request;

use std::{any::TypeId, collections::HashMap, sync::Arc};

pub use builder::*;
use graphql_mocks::{MockGraphQlServer, ReceivedRequest};
pub use request::*;

use crate::engine_v1::GraphQlRequest;

pub struct TestEngineV2 {
    engine: Arc<engine_v2::Engine<TestRuntime>>,
    subgraphs: HashMap<TypeId, Subgraph>,
}

pub struct Subgraph {
    pub name: String,
    pub server: MockGraphQlServer,
}

impl std::ops::Deref for Subgraph {
    type Target = MockGraphQlServer;
    fn deref(&self) -> &Self::Target {
        &self.server
    }
}

impl std::ops::DerefMut for Subgraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.server
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

    pub fn subgraph<S: graphql_mocks::Subgraph>(&self) -> &Subgraph {
        self.subgraphs.get(&std::any::TypeId::of::<S>()).unwrap()
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
