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
    execution::{ExecutionContext, ExecutionError, ExecutionResult, OperationRootPlanExecution},
    operation::OperationType,
    plan::{ExecutionPlan, PlanWalker, PlanningResult},
    response::{ResponseObjectsView, ResponsePart},
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
    Unreachable, // FIXME: replace by an executor which always errors.
    GraphQL(GraphqlPreparedExecutor),
    FederationEntity(FederationEntityPreparedExecutor),
    Introspection(IntrospectionPreparedExecutor),
}

impl PreparedExecutor {
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

pub(crate) struct ExecutorInput<'ctx, 'input> {
    pub ctx: ExecutionContext<'ctx>,
    pub plan: PlanWalker<'ctx, (), ()>,
    pub root_response_objects: ResponseObjectsView<'input>,
}

pub(crate) struct SubscriptionInput<'ctx> {
    pub ctx: ExecutionContext<'ctx>,
    pub plan: PlanWalker<'ctx>,
}

impl ExecutionPlan {
    pub fn new_executor<'ctx>(&'ctx self, input: ExecutorInput<'ctx, '_>) -> Result<Executor<'ctx>, ExecutionError> {
        match &self.prepared_executor {
            PreparedExecutor::Unreachable => unreachable!(),
            PreparedExecutor::Introspection(prepared) => prepared.new_executor(input),
            PreparedExecutor::GraphQL(prepared) => prepared.new_executor(input),
            PreparedExecutor::FederationEntity(prepared) => prepared.new_executor(input),
        }
    }

    pub fn new_subscription_executor<'ctx>(
        &'ctx self,
        input: SubscriptionInput<'ctx>,
    ) -> Result<SubscriptionExecutor<'ctx>, ExecutionError> {
        match &self.prepared_executor {
            PreparedExecutor::Unreachable => unreachable!(),
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

pub(crate) enum Executor<'ctx> {
    GraphQL(GraphqlExecutor<'ctx>),
    Introspection(IntrospectionExecutor<'ctx>),
    FederationEntity(FederationEntityExecutor<'ctx>),
}

impl<'ctx> Executor<'ctx> {
    pub async fn execute(self, response_part: ResponsePart) -> ExecutionResult<ResponsePart> {
        match self {
            Executor::GraphQL(executor) => executor.execute(response_part).await,
            Executor::Introspection(executor) => executor.execute(response_part).await,
            Executor::FederationEntity(executor) => executor.execute(response_part).await,
        }
    }
}

pub(crate) enum SubscriptionExecutor<'ctx> {
    Graphql(GraphqlSubscriptionExecutor<'ctx>),
}

impl<'ctx> SubscriptionExecutor<'ctx> {
    pub async fn execute(
        self,
        new_execution: impl Fn() -> OperationRootPlanExecution<'ctx> + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<OperationRootPlanExecution<'ctx>>>> {
        match self {
            SubscriptionExecutor::Graphql(executor) => executor.execute(new_execution).await,
        }
    }
}
