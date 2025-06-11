mod extension;
mod graphql;
mod introspection;
mod lookup;

pub(crate) use extension::{ExtensionResolver, FieldResolverExtension, SelectionSetExtensionResolver};
use futures::{FutureExt, future::BoxFuture};
use futures_util::stream::BoxStream;
pub(crate) use graphql::{FederationEntityResolver, GraphqlResolver};
use introspection::IntrospectionResolver;
pub(crate) use lookup::{LookupProxiedResolver, LookupResolver};
use operation::{Operation, OperationContext};
use runtime::hooks::Hooks;
use schema::ResolverDefinitionVariant;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{Plan, PlanQueryPartition, PlanResult, PrepareContext},
    response::{ParentObjects, ResponseBuilder, ResponsePartBuilder},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum Resolver {
    Graphql(GraphqlResolver),
    FederationEntity(FederationEntityResolver),
    Introspection(IntrospectionResolver),
    FieldResolverExtension(FieldResolverExtension),
    SelectionSetExtension(SelectionSetExtensionResolver),
    Extension(ExtensionResolver),
    Lookup(LookupResolver),
}

impl Resolver {
    pub async fn prepare(
        ctx: &PrepareContext<'_, impl Runtime>,
        // TODO: Should be part of the context in some way.
        operation: &Operation,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> PlanResult<Self> {
        let opctx = OperationContext {
            schema: ctx.schema(),
            operation,
        };
        match plan_query_partition.resolver_definition().variant() {
            ResolverDefinitionVariant::Introspection(_) => Ok(Resolver::Introspection(IntrospectionResolver)),
            ResolverDefinitionVariant::GraphqlRootField(definition) => {
                GraphqlResolver::prepare(opctx, definition, plan_query_partition.selection_set()).map(Self::Graphql)
            }
            ResolverDefinitionVariant::GraphqlFederationEntity(definition) => {
                FederationEntityResolver::prepare(opctx, definition, plan_query_partition).map(Self::FederationEntity)
            }
            ResolverDefinitionVariant::FieldResolverExtension(definition) => {
                FieldResolverExtension::prepare(ctx, definition, plan_query_partition.selection_set())
                    .await
                    .map(Self::FieldResolverExtension)
            }
            ResolverDefinitionVariant::Extension(definition) => {
                ExtensionResolver::prepare(ctx, definition, plan_query_partition.selection_set())
                    .await
                    .map(Self::Extension)
            }
            ResolverDefinitionVariant::SelectionSetResolverExtension(definition) => {
                SelectionSetExtensionResolver::prepare(ctx, definition, plan_query_partition.selection_set())
                    .await
                    .map(Self::SelectionSetExtension)
            }
            ResolverDefinitionVariant::Lookup(definition) => {
                LookupResolver::prepare(ctx, operation, definition, plan_query_partition)
                    .await
                    .map(Self::Lookup)
            }
        }
    }
}

pub struct ResolverResult<'ctx, OnSubgraphResponseHookOutput> {
    pub response_part: ResponsePartBuilder<'ctx>,
    pub on_subgraph_response_hook_output: Option<OnSubgraphResponseHookOutput>,
}

impl Resolver {
    pub fn execute<'ctx, 'fut, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        // This cannot be kept in the future, it locks the whole the response to have this view.
        // So an executor is expected to prepare whatever it required from the response before
        // awaiting anything.
        parent_objects_view: ParentObjects<'_>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> BoxFuture<'fut, ResolverResult<'ctx, <R::Hooks as Hooks>::OnSubgraphResponseOutput>>
    where
        'ctx: 'fut,
    {
        match self {
            Resolver::Graphql(prepared) => {
                let parent_objects = parent_objects_view.into_object_set();
                async move {
                    let mut ctx = prepared.build_subgraph_context(ctx);
                    let subgraph_result = prepared.execute(&mut ctx, plan, parent_objects, response_part).await;
                    ctx.finalize(subgraph_result).await
                }
            }
            .boxed(),
            Resolver::FederationEntity(prepared) => {
                let mut ctx = prepared.build_subgraph_context(ctx);
                let executor = prepared.build_executor(&ctx, plan, parent_objects_view, response_part);

                async move {
                    let part = executor.execute(&mut ctx).await;
                    ctx.finalize(part).await
                }
                .boxed()
            }
            Resolver::Introspection(prepared) => {
                let parent_objects = parent_objects_view.into_object_set();
                async move {
                    let response_part = prepared.execute(ctx, plan, parent_objects, response_part);

                    ResolverResult {
                        response_part,
                        on_subgraph_response_hook_output: None,
                    }
                }
            }
            .boxed(),
            Resolver::FieldResolverExtension(prepared) => {
                let executor = prepared.build_executor(ctx, plan, parent_objects_view, response_part);
                async move {
                    ResolverResult {
                        response_part: executor.execute().await,
                        on_subgraph_response_hook_output: None,
                    }
                }
                .boxed()
            }
            Resolver::Extension(prepared) => {
                let fut = prepared.execute(ctx, plan, parent_objects_view, response_part);
                async move {
                    let response_part = fut.await;
                    ResolverResult {
                        response_part,
                        on_subgraph_response_hook_output: None,
                    }
                }
                .boxed()
            }
            Resolver::SelectionSetExtension(prepared) => {
                let fut = prepared.execute(ctx, plan, parent_objects_view.into_object_set(), response_part);
                async move {
                    let response_part = fut.await;
                    ResolverResult {
                        response_part,
                        on_subgraph_response_hook_output: None,
                    }
                }
                .boxed()
            }
            Resolver::Lookup(resolver) => resolver.execute(ctx, plan, parent_objects_view, response_part),
        }
    }

    pub async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        new_response: impl Fn() -> ResponseBuilder<'ctx> + Send + Copy + 'ctx,
    ) -> BoxStream<'ctx, (ResponseBuilder<'ctx>, ResponsePartBuilder<'ctx>)> {
        match self {
            Resolver::Graphql(resolver) => {
                // TODO: for now we do not finalize this, e.g. we do not call the subgraph response hook. We should figure
                // out later what kind of data that hook would contain.
                let mut ctx = resolver.build_subgraph_context(ctx);
                resolver.execute_subscription(&mut ctx, plan, new_response).await
            }
            Resolver::FieldResolverExtension(resolver) => resolver.execute_subscription(ctx, plan, new_response).await,
            Resolver::Extension(resolver) => resolver.execute_subscription(ctx, plan, new_response).await,
            Resolver::Lookup(_)
            | Resolver::SelectionSetExtension(_)
            | Resolver::Introspection(_)
            | Resolver::FederationEntity(_) => {
                unreachable!("Unsupported subscription resolver")
            }
        }
    }
}
