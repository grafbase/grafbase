use futures_util::stream::BoxStream;
use schema::{Resolver, ResolverWalker};

use crate::{
    execution::ExecutionContext,
    plan::{PlanBoundary, PlanId, PlanOutput},
    request::EntityType,
    response::{ExecutorOutput, GraphqlError, ResponseBoundaryObjectsView, ResponseBuilder},
};

mod graphql;
mod introspection;

use graphql::federation::FederationEntityExecutor;
use graphql::GraphqlExecutor;
use introspection::IntrospectionExecutionPlan;

use self::graphql::GraphqlSubscriptionExecutor;

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
        match walker.as_ref() {
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

    pub fn plan_id(&self) -> PlanId {
        match self {
            Executor::GraphQL(executor) => executor.plan_id,
            Executor::Introspection(executor) => executor.plan_id,
            Executor::FederationEntity(executor) => executor.plan_id,
        }
    }
}

#[allow(dead_code)]
pub(crate) struct SubscriptionResolverInput<'ctx> {
    pub ctx: ExecutionContext<'ctx>,
    pub plan_id: PlanId,
    pub plan_output: PlanOutput,
    pub plan_boundaries: Vec<PlanBoundary>,
}

#[allow(dead_code)]
pub(crate) enum SubscriptionExecutor<'a> {
    Graphql(GraphqlSubscriptionExecutor<'a>),
}

impl<'exc> SubscriptionExecutor<'exc> {
    pub fn build<'ctx>(
        walker: ResolverWalker<'ctx>,
        _entity_type: EntityType,
        input: SubscriptionResolverInput<'ctx>,
    ) -> ExecutorResult<Self>
    where
        'ctx: 'exc,
    {
        match walker.as_ref() {
            Resolver::FederationRootField(resolver) => GraphqlSubscriptionExecutor::build(walker.walk(resolver), input),
            Resolver::Introspection(_) => Err(ExecutorError::Internal(
                "Subscriptions can't contain introspection".into(),
            )),
            Resolver::FederationEntity(_) => Err(ExecutorError::Internal(
                "Subscriptions can only be at the root of a query so can't contain federated entitites".into(),
            )),
        }
    }

    pub async fn execute(self) -> ExecutorResult<BoxStream<'exc, (ResponseBuilder, ExecutorOutput)>> {
        match self {
            SubscriptionExecutor::Graphql(executor) => executor.execute().await,
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
            ..Default::default()
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
