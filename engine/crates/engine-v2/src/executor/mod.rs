use schema::Resolver;

use crate::{
    execution::ExecutionContext,
    response::{ResponseObjectsView, ResponsePartBuilder},
};

mod graphql;
mod introspection;

use graphql::GraphqlExecutor;
use introspection::IntrospectionExecutor;

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
        ctx: ExecutionContext<'ctx, 'ctx>,
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

    pub async fn execute(
        self,
        ctx: ExecutionContext<'_, '_>,
        output: &mut ResponsePartBuilder,
    ) -> Result<(), ExecutorError> {
        match self {
            Executor::GraphQL(executor) => executor.execute(ctx, output).await,
            Executor::Introspection(executor) => executor.execute(ctx, output).await,
        }
    }
}

pub struct ExecutorInput<'a> {
    pub root_response_objects: ResponseObjectsView<'a>,
}

#[derive(thiserror::Error, Debug)]
pub enum ExecutorError {
    #[error("Internal error: {0}")]
    Internal(String),
    #[error(transparent)]
    Write(#[from] crate::response::WriteError),
    #[error(transparent)]
    Fetch(#[from] runtime::fetch::FetchError),
}

impl From<&str> for ExecutorError {
    fn from(message: &str) -> Self {
        Self::Internal(message.to_string())
    }
}

impl From<String> for ExecutorError {
    fn from(message: String) -> Self {
        Self::Internal(message)
    }
}
