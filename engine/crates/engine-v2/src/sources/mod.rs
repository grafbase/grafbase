//! Execution plans are responsible to retrieve a selection_set from a certain point in the query.
//!
//! Supposing we have a query like this:
//! ```graphql
//! query {
//!     catalog {
//!         products {
//!             name
//!             price
//!         }
//!     }
//! }
//! ```
//!
//! If `prices` comes from a different data source we would have two plans like:
//! ```graphql
//! # Catalog plan
//! query {
//!     catalog {
//!         products {
//!             id
//!             name
//!         }
//!     }
//! }
//! ```
//!
//! ```graphql
//! # Price plan
//! query {
//!      _entities(representations: [...]) {
//!         ... on Product {
//!             price
//!         }
//!     }
//! }
//! ```
//!
//! Execution plans define what to do at runtime for a given query. They only depend on the
//! operation and thus can be cached and do not depend on any context. On the other hand,
//! Executors are context (variables, response, headers, etc.) depend and built from the execution plans
//!
//! The executor for the catalog plan would have a single response object root and the price plan
//! executor will have a root for each product in the response.
use futures_util::stream::BoxStream;
use schema::{Resolver, ResolverWalker};

use crate::{
    execution::{ExecutionContext, ExecutionError, ExecutionResult, PlanWalker, PlanningResult, SubscriptionResponse},
    operation::OperationType,
    response::{ResponseObjectsView, SubgraphResponseMutRef},
    Runtime,
};

use self::{
    graphql::{
        FederationEntityExecutor, FederationEntityPreparedExecutor, GraphqlExecutor, GraphqlPreparedExecutor,
        GraphqlSubscriptionExecutor,
    },
    introspection::{IntrospectionExecutor, IntrospectionPreparedExecutor},
};

mod graphql;
mod introspection;

pub(crate) enum PreparedExecutor {
    GraphQL(GraphqlPreparedExecutor),
    FederationEntity(FederationEntityPreparedExecutor),
    Introspection(IntrospectionPreparedExecutor),
}

impl PreparedExecutor {
    pub fn introspection() -> Self {
        PreparedExecutor::Introspection(IntrospectionPreparedExecutor)
    }

    pub fn prepare(
        walker: ResolverWalker<'_>,
        operation_type: OperationType,
        plan: PlanWalker<'_>,
    ) -> PlanningResult<Self> {
        match walker.as_ref() {
            Resolver::Introspection(_) => Ok(PreparedExecutor::Introspection(IntrospectionPreparedExecutor)),
            Resolver::GraphqlRootField(resolver) => {
                GraphqlPreparedExecutor::prepare(walker.walk(resolver), operation_type, plan)
            }
            Resolver::GraphqlFederationEntity(resolver) => {
                FederationEntityPreparedExecutor::prepare(walker.walk(resolver), plan)
            }
        }
    }
}

pub(crate) struct ExecutorInput<'ctx, 'input, R: Runtime> {
    pub ctx: ExecutionContext<'ctx, R>,
    pub plan: PlanWalker<'ctx, (), ()>,
    pub root_response_objects: ResponseObjectsView<'input>,
}

pub(crate) struct SubscriptionInput<'ctx, R: Runtime> {
    pub ctx: ExecutionContext<'ctx, R>,
    pub plan: PlanWalker<'ctx>,
}

impl PreparedExecutor {
    pub fn new_executor<'ctx, R: Runtime>(
        &'ctx self,
        input: ExecutorInput<'ctx, '_, R>,
    ) -> Result<Executor<'ctx, R>, ExecutionError> {
        match self {
            PreparedExecutor::Introspection(prepared) => prepared.new_executor(input),
            PreparedExecutor::GraphQL(prepared) => prepared.new_executor(input),
            PreparedExecutor::FederationEntity(prepared) => prepared.new_executor(input),
        }
    }

    pub fn new_subscription_executor<'ctx, R: Runtime>(
        &'ctx self,
        input: SubscriptionInput<'ctx, R>,
    ) -> Result<SubscriptionExecutor<'ctx, R>, ExecutionError> {
        match self {
            PreparedExecutor::GraphQL(prepared) => prepared.new_subscription_executor(input),
            PreparedExecutor::Introspection(_) => Err(ExecutionError::Internal(
                "Subscriptions can't contain introspection".into(),
            )),
            PreparedExecutor::FederationEntity(_) => Err(ExecutionError::Internal(
                "Subscriptions can only be at the root of a query so can't contain federated entitites".into(),
            )),
        }
    }
}

pub(crate) enum Executor<'ctx, R: Runtime> {
    GraphQL(GraphqlExecutor<'ctx, R>),
    Introspection(IntrospectionExecutor<'ctx, R>),
    FederationEntity(FederationEntityExecutor<'ctx, R>),
}

impl<'ctx, R: Runtime> Executor<'ctx, R> {
    pub async fn execute<'resp>(self, subgraph_response: SubgraphResponseMutRef<'resp>) -> ExecutionResult<()>
    where
        'ctx: 'resp,
    {
        match self {
            Executor::GraphQL(executor) => executor.execute(subgraph_response).await,
            Executor::Introspection(executor) => executor.execute(subgraph_response).await,
            Executor::FederationEntity(executor) => executor.execute(subgraph_response).await,
        }
    }
}

pub(crate) enum SubscriptionExecutor<'ctx, R: Runtime> {
    Graphql(GraphqlSubscriptionExecutor<'ctx, R>),
}

impl<'ctx, R: Runtime> SubscriptionExecutor<'ctx, R> {
    pub async fn execute(
        self,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        match self {
            SubscriptionExecutor::Graphql(executor) => executor.execute(new_response).await,
        }
    }
}
