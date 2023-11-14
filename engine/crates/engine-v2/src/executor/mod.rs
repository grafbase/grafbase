use std::sync::Arc;

use engine_parser::types::OperationType;
use futures_locks::Mutex;
use schema::Resolver;

use crate::{
    request::OperationFields,
    response::{Response, ResponseObjectsView, WriteSelectionSet},
    Engine,
};

mod coordinator;
mod graphql;

pub use coordinator::ExecutorCoordinator;

use self::graphql::GraphqlExecutor;

struct ExecutorRequest<'a> {
    operation_type: OperationType,
    operation_fields: &'a OperationFields,
    response_objects: ResponseObjectsView<'a>,
    output: &'a WriteSelectionSet,
}

enum Executor {
    GraphQL(GraphqlExecutor),
}

#[derive(thiserror::Error, Debug)]
enum ExecutorError {
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<&str> for ExecutorError {
    fn from(message: &str) -> Self {
        Self::InternalError(message.to_string())
    }
}

impl From<String> for ExecutorError {
    fn from(message: String) -> Self {
        Self::InternalError(message)
    }
}

impl Executor {
    fn build(engine: &Engine, resolver: &schema::Resolver, request: ExecutorRequest<'_>) -> Self {
        match resolver {
            Resolver::Subgraph(resolver) => GraphqlExecutor::build(engine, resolver, request),
        }
    }

    async fn execute(self, response: ResponseProxy) -> Result<(), ExecutorError> {
        match self {
            Executor::GraphQL(executor) => executor.execute(response).await,
        }
    }
}

struct ResponseProxy {
    inner: Arc<Mutex<Response>>,
}

impl ResponseProxy {
    // Need something cleaner here. Ideally I just want something that makes it not too easy
    // to hold the lock indefinitely.
    // Guaranteed to be executed before any children.
    async fn mutate<T>(&self, func: impl FnOnce(&mut Response) -> T) -> T {
        let mut graph = self.inner.lock().await;
        func(&mut graph)
    }
}
