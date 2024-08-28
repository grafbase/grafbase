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
use runtime::hooks::ExecutedSubgraphRequest;
use schema::{sources::graphql::GraphqlEndpointWalker, ResolverDefinition, ResolverDefinitionWalker};
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
    pub fn introspection() -> Self {
        Resolver::Introspection(IntrospectionResolver)
    }

    pub fn prepare(
        definition: ResolverDefinitionWalker<'_>,
        operation_type: OperationType,
        plan: PlanWalker<'_>,
    ) -> PlanningResult<Self> {
        match definition.as_ref() {
            ResolverDefinition::Introspection(_) => Ok(Resolver::Introspection(IntrospectionResolver)),
            ResolverDefinition::GraphqlRootField(resolver) => {
                GraphqlResolver::prepare(definition.walk(resolver), operation_type, plan)
            }
            ResolverDefinition::GraphqlFederationEntity(resolver) => {
                FederationEntityResolver::prepare(definition.walk(resolver), plan)
            }
        }
    }

    pub fn endpoint<'ctx, R: Runtime>(&self, ctx: ExecutionContext<'ctx, R>) -> Option<GraphqlEndpointWalker<'ctx>> {
        match self {
            Resolver::GraphQL(ref prepared) => Some(prepared.endpoint(ctx)),
            Resolver::FederationEntity(ref prepared) => Some(prepared.endpoint(ctx)),
            Resolver::Introspection(_) => None,
        }
    }
}

pub struct ExecutionFutureResult {
    pub result: ExecutionResult<SubgraphResponse>,
    pub on_subgraph_response_hook_result: Option<Vec<u8>>,
}

impl Resolver {
    pub fn execute<'ctx, 'fut, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx, (), ()>,
        // This cannot be kept in the future, it locks the whole the response to have this view.
        // So an executor is expected to prepare whatever it required from the response before
        // awaiting anything.
        root_response_objects: ResponseObjectsView<'fut>,
        subgraph_response: SubgraphResponse,
    ) -> impl Future<Output = ExecutionFutureResult> + Send + 'fut
    where
        'ctx: 'fut,
    {
        match self {
            Resolver::GraphQL(prepared) => {
                let hooks = ctx.hooks();

                async move {
                    let endpoint = prepared.endpoint(ctx);

                    let mut request_info =
                        ExecutedSubgraphRequest::builder(endpoint.subgraph_name(), "POST", endpoint.url().as_str());

                    let result = prepared.execute(ctx, plan, subgraph_response, &mut request_info).await;
                    let hook_result = hooks.on_subgraph_response(request_info.build()).await.unwrap();

                    ExecutionFutureResult {
                        result,
                        on_subgraph_response_hook_result: Some(hook_result),
                    }
                }
                .boxed()
            }
            Resolver::FederationEntity(prepared) => {
                let hooks = ctx.hooks();

                async move {
                    let endpoint = prepared.endpoint(ctx);

                    let mut request_info =
                        ExecutedSubgraphRequest::builder(endpoint.subgraph_name(), "POST", endpoint.url().as_str());

                    let result = prepared
                        .execute(ctx, plan, root_response_objects, subgraph_response, &mut request_info)
                        .await;

                    let hook_result = hooks.on_subgraph_response(request_info.build()).await.unwrap();

                    ExecutionFutureResult {
                        result,
                        on_subgraph_response_hook_result: Some(hook_result),
                    }
                }
                .boxed()
            }
            Resolver::Introspection(prepared) => async move {
                let result = prepared.execute(ctx, plan, subgraph_response).await;

                ExecutionFutureResult {
                    result,
                    on_subgraph_response_hook_result: None,
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
            Resolver::GraphQL(prepared) => prepared.execute_subscription(ctx, plan, new_response).await,
            Resolver::Introspection(_) => Err(ExecutionError::Internal(
                "Subscriptions can't contain introspection".into(),
            )),
            Resolver::FederationEntity(_) => Err(ExecutionError::Internal(
                "Subscriptions can only be at the root of a query so can't contain federated entitites".into(),
            )),
        }
    }
}
