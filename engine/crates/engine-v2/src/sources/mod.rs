/// Execution plans are responsible to retrieve a selection_set from a certain point in the query.
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
///
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
///
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
/// Execution plans define what to do at runtime for a given query. They only depend on the
/// operation and thus can be cached and do not depend on any context. On the other hand,
/// Executors are context (variables, response, headers, etc.) depend and built from the execution plans
///
/// The executor for the catalog plan would have a single response object root and the price plan
/// executor will have a root for each product in the response.
use futures_util::stream::BoxStream;
use schema::{Resolver, ResolverWalker};

use crate::{
    execution::{ExecutionContext, ExecutionError, ExecutionResult},
    plan::{PlanWalker, PlanningResult},
    response::{ResponseBoundaryObjectsView, ResponseBuilder, ResponsePart},
};

use self::{
    graphql::{
        FederationEntityExecutionPlan, FederationEntityExecutor, GraphqlExecutionPlan, GraphqlExecutor,
        GraphqlSubscriptionExecutor,
    },
    introspection::{IntrospectionExecutionPlan, IntrospectionExecutor},
};

mod graphql;
mod introspection;

pub(crate) enum ExecutionPlan {
    GraphQL(GraphqlExecutionPlan),
    FederationEntity(FederationEntityExecutionPlan),
    Introspection(IntrospectionExecutionPlan),
}

impl ExecutionPlan {
    pub fn build(walker: ResolverWalker<'_>, plan: PlanWalker<'_>) -> PlanningResult<Self> {
        match walker.as_ref() {
            Resolver::Introspection(_) => Ok(ExecutionPlan::Introspection(IntrospectionExecutionPlan)),
            Resolver::FederationRootField(resolver) => GraphqlExecutionPlan::build(walker.walk(resolver), plan),
            Resolver::FederationEntity(resolver) => FederationEntityExecutionPlan::build(walker.walk(resolver), plan),
        }
    }
}

pub(crate) struct ExecutorInput<'ctx, 'input> {
    pub ctx: ExecutionContext<'ctx>,
    pub plan: PlanWalker<'ctx>,
    pub boundary_objects_view: ResponseBoundaryObjectsView<'input>,
    pub response_part: ResponsePart,
}

pub(crate) struct SubscriptionInput<'ctx> {
    pub ctx: ExecutionContext<'ctx>,
    pub plan: PlanWalker<'ctx>,
}

impl ExecutionPlan {
    pub fn new_executor<'ctx>(&'ctx self, input: ExecutorInput<'ctx, '_>) -> Result<Executor<'ctx>, ExecutionError> {
        match self {
            ExecutionPlan::Introspection(execution_plan) => execution_plan.new_executor(input),
            ExecutionPlan::GraphQL(execution_plan) => execution_plan.new_executor(input),
            ExecutionPlan::FederationEntity(execution_plan) => execution_plan.new_executor(input),
        }
    }

    pub fn new_subscription_executor<'ctx>(
        &'ctx self,
        input: SubscriptionInput<'ctx>,
    ) -> Result<SubscriptionExecutor<'ctx>, ExecutionError> {
        match self {
            ExecutionPlan::GraphQL(execution_plan) => execution_plan.new_subscription_executor(input),
            ExecutionPlan::Introspection(_) => Err(ExecutionError::Internal(
                "Subscriptions can't contain introspection".into(),
            )),
            ExecutionPlan::FederationEntity(_) => Err(ExecutionError::Internal(
                "Subscriptions can only be at the root of a query so can't contain federated entitites".into(),
            )),
        }
    }
}

pub(crate) enum Executor<'ctx> {
    GraphQL(GraphqlExecutor<'ctx>),
    Introspection(IntrospectionExecutor<'ctx>),
    FederationEntity(FederationEntityExecutor<'ctx>),
}

impl<'ctx> Executor<'ctx> {
    pub async fn execute(self) -> ExecutionResult<ResponsePart> {
        match self {
            Executor::GraphQL(executor) => executor.execute().await,
            Executor::Introspection(executor) => executor.execute().await,
            Executor::FederationEntity(executor) => executor.execute().await,
        }
    }
}

pub(crate) enum SubscriptionExecutor<'ctx> {
    Graphql(GraphqlSubscriptionExecutor<'ctx>),
}

impl<'exc> SubscriptionExecutor<'exc> {
    pub async fn execute(self) -> ExecutionResult<BoxStream<'exc, (ResponseBuilder, ResponsePart)>> {
        match self {
            SubscriptionExecutor::Graphql(executor) => executor.execute().await,
        }
    }
}
