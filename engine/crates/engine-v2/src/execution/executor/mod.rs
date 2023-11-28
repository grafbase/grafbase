use std::sync::Arc;

use futures_locks::Mutex;
use schema::Resolver;

use crate::response::{GraphqlErrors, ResponseData, ResponseObjectsView};

mod graphql;
mod introspection;

use graphql::GraphqlExecutor;
use introspection::IntrospectionExecutor;

use super::ExecutionContext;

/// Executors are responsible to retrieve a selection_set from a certain point in the query.
///
/// Supposing we have a query like this:
/// ```graphql
/// query {
///     catalog {
///         products {
///             name
///             price
///         }
///     }
/// }
/// ```
/// If `prices` comes from a different data source we would have two plans like:
/// ```graphql
/// # Catalog plan
/// query {
///     catalog {
///         products {
///             id
///             name
///         }
///     }
/// }
/// ```
/// ```graphql
/// # Price plan
/// query {
///      _entities(representations: [...]) {
///         ... on Product {
///             price
///         }
///     }
/// }
/// ```
///
/// The executor for the catalog plan would have a single response object root and the price plan
/// executor will have a root for each product in the response.
pub enum Executor<'a> {
    GraphQL(GraphqlExecutor<'a>),
    Introspection(IntrospectionExecutor<'a>),
}

impl<'exc> Executor<'exc> {
    pub fn build<'ctx, 'input>(
        ctx: ExecutionContext<'ctx>,
        resolver: &schema::Resolver,
        input: ExecutorInput<'input>,
    ) -> Result<Self, ExecutorError>
    where
        'ctx: 'exc,
    {
        match resolver {
            Resolver::Subgraph(resolver) => GraphqlExecutor::build(ctx, resolver, input),
            Resolver::Introspection(resolver) => IntrospectionExecutor::build(ctx, resolver, input),
        }
    }

    pub async fn execute(self, ctx: ExecutionContext<'_>, output: &mut ExecutorOutput) -> Result<(), ExecutorError> {
        match self {
            Executor::GraphQL(executor) => executor.execute(ctx, output).await,
            Executor::Introspection(executor) => executor.execute(ctx, output).await,
        }
    }
}

pub struct ExecutorInput<'a> {
    pub root_response_objects: ResponseObjectsView<'a>,
}

/// Executors manipulate the response data through this struct, registering any errors (without
/// locking) and modifying the actual data when necessary. Will be tweaked later to reduce lock
/// contention.
pub struct ExecutorOutput {
    pub data: Arc<Mutex<ResponseData>>,
    pub errors: GraphqlErrors,
}

#[derive(thiserror::Error, Debug)]
pub enum ExecutorError {
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
