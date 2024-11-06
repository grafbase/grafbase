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
use futures::FutureExt;
use futures_util::stream::BoxStream;
use runtime::hooks::Hooks;
use schema::{ResolverDefinition, ResolverDefinitionVariant};
use std::future::Future;

use crate::{
    execution::{ExecutionContext, ExecutionError, ExecutionResult, PlanningResult, SubscriptionResponse},
    operation::{OperationType, PlanWalker},
    response::{ResponseObjectsView, SubgraphResponse},
    Runtime,
};

use self::{
    graphql::{FederationEntityResolver, GraphqlResolver},
    introspection::IntrospectionResolver,
};

mod graphql;
mod introspection;

pub(crate) enum Resolver {
    GraphQL(GraphqlResolver),
    FederationEntity(FederationEntityResolver),
    Introspection(IntrospectionResolver),
}

impl Resolver {
    pub fn prepare(
        definition: ResolverDefinition<'_>,
        operation_type: OperationType,
        plan: PlanWalker<'_>,
    ) -> PlanningResult<Self> {
        match definition.variant() {
            ResolverDefinitionVariant::Introspection => Ok(Resolver::Introspection(IntrospectionResolver)),
            ResolverDefinitionVariant::GraphqlRootField(resolver) => {
                GraphqlResolver::prepare(resolver, operation_type, plan)
            }
            ResolverDefinitionVariant::GraphqlFederationEntity(resolver) => {
                FederationEntityResolver::prepare(resolver, plan)
            }
        }
    }
}

pub struct ResolverResult<OnSubgraphResponseHookOutput> {
    pub execution: ExecutionResult<SubgraphResponse>,
    pub on_subgraph_response_hook_output: Option<OnSubgraphResponseHookOutput>,
}

impl Resolver {
    pub fn execute<'ctx, 'fut, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx, ()>,
        // This cannot be kept in the future, it locks the whole the response to have this view.
        // So an executor is expected to prepare whatever it required from the response before
        // awaiting anything.
        root_response_objects: ResponseObjectsView<'_>,
        subgraph_response: SubgraphResponse,
    ) -> impl Future<Output = ResolverResult<<R::Hooks as Hooks>::OnSubgraphResponseOutput>> + Send + 'fut
    where
        'ctx: 'fut,
    {
        match self {
            Resolver::GraphQL(prepared) => async move {
                let mut ctx = prepared.build_subgraph_context(ctx);
                let subgraph_result = prepared.execute(&mut ctx, plan, subgraph_response).await;
                ctx.finalize(subgraph_result).await
            }
            .boxed(),
            Resolver::FederationEntity(prepared) => {
                let mut ctx = prepared.build_subgraph_context(ctx);
                let request = prepared.prepare_request(&ctx, plan, root_response_objects, subgraph_response);

                async move {
                    let subgraph_result = match request {
                        Ok(request) => request.execute(&mut ctx).await,
                        Err(error) => Err(error),
                    };

                    ctx.finalize(subgraph_result).await
                }
                .boxed()
            }
            Resolver::Introspection(prepared) => async move {
                let result = prepared.execute(ctx, plan, subgraph_response).await;

                ResolverResult {
                    execution: result,
                    on_subgraph_response_hook_output: None,
                }
            }
            .boxed(),
        }
    }

    pub async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx>,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        match self {
            Resolver::GraphQL(prepared) => {
                // TODO: for now we do not finalize this, e.g. we do not call the subgraph response hook. We should figure
                // out later what kind of data that hook would contain.
                let mut ctx = prepared.build_subgraph_context(ctx);
                prepared.execute_subscription(&mut ctx, plan, new_response).await
            }
            Resolver::Introspection(_) => Err(ExecutionError::Internal(
                "Subscriptions can't contain introspection".into(),
            )),
            Resolver::FederationEntity(_) => Err(ExecutionError::Internal(
                "Subscriptions can only be at the root of a query so can't contain federated entitites".into(),
            )),
        }
    }
}
