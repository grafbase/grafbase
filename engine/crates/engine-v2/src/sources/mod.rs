use std::collections::HashMap;

use schema::{Resolver, ResolverWalker};

use crate::{
    execution::ExecutionContext,
    plan::{PlanId, PlanOutput},
    request::EntityType,
    response::{ExecutorOutput, GraphqlError, ResponseBoundaryObjectsView},
};

mod graphql;
mod introspection;

use graphql::federation::FederationEntityExecutor;
use graphql::GraphqlExecutor;
use introspection::IntrospectionExecutionPlan;

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
pub(crate) enum Executor<'a> {
    GraphQL(GraphqlExecutor<'a>),
    Introspection(IntrospectionExecutionPlan<'a>),
    FederationEntity(FederationEntityExecutor<'a>),
}

pub(crate) struct ResolverInput<'ctx, 'input> {
    pub ctx: ExecutionContext<'ctx>,
    pub boundary_objects_view: ResponseBoundaryObjectsView<'input>,
    pub plan_id: PlanId,
    pub plan_output: PlanOutput,
    pub output: ExecutorOutput,
}

impl<'exc> Executor<'exc> {
    pub fn build<'ctx, 'input>(
        walker: ResolverWalker<'ctx>,
        entity_type: EntityType,
        input: ResolverInput<'ctx, 'input>,
    ) -> ExecutorResult<Self>
    where
        'ctx: 'exc,
    {
        match walker.get() {
            Resolver::Introspection(resolver) => IntrospectionExecutionPlan::build(walker.walk(resolver), input),
            Resolver::FederationRootField(resolver) => GraphqlExecutor::build(walker.walk(resolver), input),
            Resolver::FederationEntity(resolver) => {
                FederationEntityExecutor::build(walker.walk(resolver), entity_type, input)
            }
        }
    }

    pub async fn execute(self) -> ExecutorResult<ExecutorOutput> {
        match self {
            Executor::GraphQL(executor) => executor.execute().await,
            Executor::Introspection(executor) => executor.execute().await,
            Executor::FederationEntity(executor) => executor.execute().await,
        }
    }
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

pub type ExecutorResult<T> = Result<T, ExecutorError>;

impl From<ExecutorError> for GraphqlError {
    fn from(err: ExecutorError) -> Self {
        GraphqlError {
            message: err.to_string(),
            locations: vec![],
            path: None,
            extensions: HashMap::with_capacity(0),
        }
    }
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
