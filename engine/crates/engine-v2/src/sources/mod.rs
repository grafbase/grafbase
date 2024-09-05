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
use runtime::hooks::{ExecutedSubgraphRequest, ExecutedSubgraphRequestBuilder};
use schema::{sources::graphql::GraphqlEndpointWalker, ResolverDefinition, ResolverDefinitionWalker};
use std::{future::Future, ops::Deref};
use tower::retry::budget::Budget;

use crate::{
    execution::{
        ExecutionContext, ExecutionError, ExecutionResult, PlanningResult, RequestHooks, SubscriptionResponse,
    },
    operation::{CacheScope, OperationType, PlanWalker},
    response::{ResponseObjectsView, SubgraphResponse},
    Engine, Runtime,
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
}

pub struct ExecutionFutureResult {
    pub result: ExecutionResult<SubgraphResponse>,
    pub on_subgraph_response_hook_result: Option<Vec<u8>>,
}

#[derive(Clone)]
pub(crate) struct SubgraphRequestContext<'ctx, R: Runtime> {
    execution_context: ExecutionContext<'ctx, R>,
    plan: PlanWalker<'ctx, (), ()>,
    request_info: ExecutedSubgraphRequestBuilder<'ctx>,
    endpoint: GraphqlEndpointWalker<'ctx>,
    retry_budget: Option<&'ctx Budget>,
}

impl<'ctx, R: Runtime> Deref for SubgraphRequestContext<'ctx, R> {
    type Target = ExecutionContext<'ctx, R>;

    fn deref(&self) -> &Self::Target {
        &self.execution_context
    }
}

impl<'ctx, R: Runtime> SubgraphRequestContext<'ctx, R> {
    pub fn new(
        execution_context: ExecutionContext<'ctx, R>,
        operation_type: OperationType,
        plan: PlanWalker<'ctx, (), ()>,
        endpoint: GraphqlEndpointWalker<'ctx>,
    ) -> Self {
        let request_info = ExecutedSubgraphRequest::builder(endpoint.subgraph_name(), "POST", endpoint.url().as_str());

        let retry_budget = match operation_type {
            OperationType::Mutation => execution_context.engine.get_retry_budget_for_mutation(endpoint.id()),
            _ => execution_context
                .engine
                .get_retry_budget_for_non_mutation(endpoint.id()),
        };

        Self {
            execution_context,
            plan,
            request_info,
            endpoint,
            retry_budget,
        }
    }

    pub fn execution_context(&self) -> ExecutionContext<'ctx, R> {
        self.execution_context
    }

    pub fn engine(&self) -> &Engine<R> {
        self.execution_context().engine
    }

    pub fn plan(&self) -> PlanWalker<'ctx, (), ()> {
        self.plan
    }

    pub fn request_info(&mut self) -> &mut ExecutedSubgraphRequestBuilder<'ctx> {
        &mut self.request_info
    }

    pub fn endpoint(&self) -> GraphqlEndpointWalker<'ctx> {
        self.endpoint
    }

    pub fn hooks(&self) -> RequestHooks<'ctx, R::Hooks> {
        self.execution_context().hooks()
    }

    pub fn cache_scopes(&self) -> Vec<String> {
        self.plan()
            .cache_scopes()
            .map(|scope| match scope {
                CacheScope::Authenticated => "authenticated".into(),
                CacheScope::RequiresScopes(scopes) => {
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(b"requiresScopes");
                    hasher.update(&scopes.scopes().len().to_le_bytes());
                    for scope in scopes.scopes() {
                        hasher.update(&scope.len().to_le_bytes());
                        hasher.update(scope.as_bytes());
                    }
                    hasher.finalize().to_hex().to_string()
                }
            })
            .collect()
    }

    pub fn retry_budget(&self) -> Option<&Budget> {
        self.retry_budget
    }

    pub async fn finalize(self, subgraph_result: ExecutionResult<SubgraphResponse>) -> ExecutionFutureResult {
        let hook_result = self
            .execution_context
            .hooks()
            .on_subgraph_response(self.request_info.build())
            .await
            .map_err(|e| {
                tracing::error!("error in on-subgraph-response hook: {e}");
                ExecutionError::Internal("internal error".into())
            });

        match hook_result {
            Ok(hook_result) => ExecutionFutureResult {
                result: subgraph_result,
                on_subgraph_response_hook_result: Some(hook_result),
            },
            Err(e) => ExecutionFutureResult {
                result: Err(e),
                on_subgraph_response_hook_result: None,
            },
        }
    }
}

impl Resolver {
    pub fn execute<'ctx, 'fut, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx, (), ()>,
        // This cannot be kept in the future, it locks the whole the response to have this view.
        // So an executor is expected to prepare whatever it required from the response before
        // awaiting anything.
        root_response_objects: ResponseObjectsView<'_>,
        subgraph_response: SubgraphResponse,
    ) -> impl Future<Output = ExecutionFutureResult> + Send + 'fut
    where
        'ctx: 'fut,
    {
        match self {
            Resolver::GraphQL(prepared) => async move {
                let mut context =
                    SubgraphRequestContext::new(ctx, prepared.operation_type(), plan, prepared.endpoint(ctx));

                let subgraph_result = prepared.execute(&mut context, subgraph_response).await;
                context.finalize(subgraph_result).await
            }
            .boxed(),
            Resolver::FederationEntity(prepared) => {
                let request = prepared.prepare_request(ctx, plan, root_response_objects, subgraph_response);

                async move {
                    let mut context =
                        SubgraphRequestContext::new(ctx, OperationType::Query, plan, prepared.endpoint(ctx));

                    let subgraph_result = match request {
                        Ok(request) => request.execute(&mut context).await,
                        Err(error) => Err(error),
                    };

                    context.finalize(subgraph_result).await
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
            Resolver::GraphQL(prepared) => {
                // TODO: for now we do not finalize this, e.g. we do not call the subgraph response hook. We should figure
                // out later what kind of data that hook would contain.
                let mut context = SubgraphRequestContext::new(ctx, OperationType::Query, plan, prepared.endpoint(ctx));
                prepared.execute_subscription(&mut context, new_response).await
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
