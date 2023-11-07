use std::sync::Arc;

use engine_parser::types::OperationType;
use futures_locks::Mutex;
use schema::Resolver;

use crate::{
    response_graph::{Input, OutputNodeSelectionSet, ResponseGraph},
    Engine,
};

mod coordinator;
mod graphql;

pub use coordinator::ExecutorCoordinator;

struct ExecutorRequest<'a> {
    operation_type: OperationType,
    input: Input<'a>,
    output: &'a OutputNodeSelectionSet,
}

enum Executor {
    GraphQL(graphql::GraphqlExecutor),
}

impl Executor {
    fn build(engine: &Engine, resolver: &schema::Resolver, request: ExecutorRequest<'_>) -> Self {
        match resolver {
            Resolver::Subgraph(resolver) => graphql::GraphqlExecutor::build(engine, resolver, request),
        }
    }

    async fn execute(self, proxy: ResponseGraphProxy) {
        match self {
            Executor::GraphQL(plan) => plan.execute(proxy).await,
        }
    }
}

struct ResponseGraphProxy {
    graph: Arc<Mutex<ResponseGraph>>,
}

impl ResponseGraphProxy {
    // Guaranteed to be executed before any children.
    async fn mutate<T>(&self, func: impl FnOnce(&mut ResponseGraph) -> T) -> T {
        let mut graph = self.graph.lock().await;
        func(&mut graph)
    }
}
