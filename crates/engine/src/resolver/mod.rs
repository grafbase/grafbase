mod extension;
mod graphql;
mod introspection;
mod lookup;

use extension::{FieldResolverExtension, SelectionSetResolverExtension};
use futures::{FutureExt, future::BoxFuture};
use futures_util::stream::BoxStream;
use graphql::{FederationEntityResolver, GraphqlResolver};
use introspection::IntrospectionResolver;
use lookup::LookupResolver;
use runtime::hooks::Hooks;
use schema::ResolverDefinitionVariant;

use crate::{
    Runtime,
    execution::{ExecutionContext, ExecutionError, ExecutionResult},
    prepare::{Plan, PlanQueryPartition, PlanResult, PrepareContext},
    response::{ParentObjectsView, ResponseBuilder, ResponsePartBuilder},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum Resolver {
    Graphql(GraphqlResolver),
    FederationEntity(FederationEntityResolver),
    Introspection(IntrospectionResolver),
    FieldResolverExtension(FieldResolverExtension),
    SelectionSetResolverExtension(SelectionSetResolverExtension),
    Lookup(LookupResolver),
}

impl Resolver {
    pub async fn prepare(
        ctx: &PrepareContext<'_, impl Runtime>,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> PlanResult<Self> {
        match plan_query_partition.resolver_definition().variant() {
            ResolverDefinitionVariant::Introspection(_) => Ok(Resolver::Introspection(IntrospectionResolver)),
            ResolverDefinitionVariant::GraphqlRootField(definition) => {
                GraphqlResolver::prepare(ctx, definition, plan_query_partition.selection_set()).map(Self::Graphql)
            }
            ResolverDefinitionVariant::GraphqlFederationEntity(definition) => {
                FederationEntityResolver::prepare(definition, plan_query_partition).map(Self::FederationEntity)
            }
            ResolverDefinitionVariant::FieldResolverExtension(definition) => {
                FieldResolverExtension::prepare(ctx, definition, plan_query_partition.selection_set())
                    .await
                    .map(Self::FieldResolverExtension)
            }
            ResolverDefinitionVariant::SelectionSetResolverExtension(definition) => {
                SelectionSetResolverExtension::prepare(ctx, definition, plan_query_partition.selection_set())
                    .await
                    .map(Self::SelectionSetResolverExtension)
            }
            ResolverDefinitionVariant::Lookup(definition) => {
                LookupResolver::prepare(ctx, definition, plan_query_partition)
                    .await
                    .map(Self::Lookup)
            }
        }
    }
}

pub struct ResolverResult<'ctx, OnSubgraphResponseHookOutput> {
    pub execution: ExecutionResult<ResponsePartBuilder<'ctx>>,
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
        parent_objects_view: ParentObjectsView<'_>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> BoxFuture<'fut, ResolverResult<'ctx, <R::Hooks as Hooks>::OnSubgraphResponseOutput>>
    where
        'ctx: 'fut,
    {
        match self {
            Resolver::Graphql(prepared) => {
                let parent_objects = parent_objects_view.into_parent_objects();
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
                    let subgraph_result = match executor {
                        Ok(executor) => executor.execute(&mut ctx).await,
                        Err(error) => Err(error),
                    };

                    ctx.finalize(subgraph_result).await
                }
                .boxed()
            }
            Resolver::Introspection(prepared) => {
                let input_object_refs = parent_objects_view.into_parent_objects();
                async move {
                    let result = prepared.execute(ctx, plan, input_object_refs, response_part);

                    ResolverResult {
                        execution: result,
                        on_subgraph_response_hook_output: None,
                    }
                }
            }
            .boxed(),
            Resolver::FieldResolverExtension(prepared) => {
                let executor = prepared.build_executor(ctx, plan, parent_objects_view, response_part);
                async move {
                    ResolverResult {
                        execution: executor.execute().await,
                        on_subgraph_response_hook_output: None,
                    }
                }
                .boxed()
            }
            Resolver::SelectionSetResolverExtension(prepared) => {
                let parent_objects = parent_objects_view.into_parent_objects();
                async move {
                    let response_part = prepared.execute(ctx, plan, parent_objects, response_part).await;
                    ResolverResult {
                        execution: Ok(response_part),
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
        new_response: impl Fn() -> ResponseBuilder<'ctx> + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<(ResponseBuilder<'ctx>, ResponsePartBuilder<'ctx>)>>> {
        match self {
            Resolver::Graphql(prepared) => {
                // TODO: for now we do not finalize this, e.g. we do not call the subgraph response hook. We should figure
                // out later what kind of data that hook would contain.
                let mut ctx = prepared.build_subgraph_context(ctx);
                prepared.execute_subscription(&mut ctx, plan, new_response).await
            }
            Resolver::FieldResolverExtension(prepared) => prepared.execute_subscription(ctx, plan, new_response).await,
            Resolver::Lookup(_)
            | Resolver::Introspection(_)
            | Resolver::FederationEntity(_)
            | Resolver::SelectionSetResolverExtension(_) => Err(ExecutionError::Internal(
                "Subscriptions are not supported by this resolver".into(),
            )),
        }
    }
}
