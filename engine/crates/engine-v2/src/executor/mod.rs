use std::sync::Arc;

use engine_parser::types::OperationType;
use futures_locks::Mutex;
use schema::Resolver;

use crate::{
    request::{OperationFields, OperationSelectionSet},
    response::{Response, ResponseObjectsView},
    Engine,
};

mod coordinator;
mod graphql;

pub use coordinator::ExecutorCoordinator;

use self::graphql::GraphqlExecutor;

struct ExecutorContext<'a> {
    // -- Common --
    operation_type: OperationType,
    operation_fields: &'a OperationFields,
    response: &'a Response,
    // -- Plan-specific --
    // On which objects inside the response this selection_set applies to.
    // All required fields will be available.
    response_object_roots: ResponseObjectsView<'a>,
    // Selection set that the executor is supposed to retrieve.
    selection_set: &'a OperationSelectionSet,
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
    fn build(engine: &Engine, resolver: &schema::Resolver, ctx: ExecutorContext<'_>) -> Result<Self, ExecutorError> {
        match resolver {
            Resolver::Subgraph(resolver) => GraphqlExecutor::build(engine, resolver, ctx),
        }
    }

    async fn execute(self, response: Arc<Mutex<Response>>) -> Result<(), ExecutorError> {
        match self {
            Executor::GraphQL(executor) => executor.execute(response).await,
        }
    }
}
