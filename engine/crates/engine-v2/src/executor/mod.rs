use std::sync::Arc;

use engine_parser::types::OperationType;
use futures_locks::Mutex;
use schema::Resolver;

use crate::{
    response::{Response, ResponseObjectsView, WriteSelectionSet},
    Engine,
};

mod coordinator;
mod graphql;

pub use coordinator::ExecutorCoordinator;

struct ExecutorRequest<'a> {
    operation_type: OperationType,
    response_objects: ResponseObjectsView<'a>,
    output: &'a WriteSelectionSet,
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

    async fn execute(self, response: ResponseProxy) {
        match self {
            Executor::GraphQL(executor) => executor.execute(response).await,
        }
    }
}

struct ResponseProxy {
    inner: Arc<Mutex<Response>>,
}

impl ResponseProxy {
    // Guaranteed to be executed before any children.
    async fn mutate<T>(&self, func: impl FnOnce(&mut Response) -> T) -> T {
        let mut graph = self.inner.lock().await;
        func(&mut graph)
    }
}
