use futures::{FutureExt as _, future::BoxFuture};
use runtime::hooks::Hooks;
use schema::{LookupResolverDefinition, ResolverDefinitionVariant};

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{Plan, PlanError, PlanQueryPartition, PlanResult, PrepareContext},
    response::{ParentObjectsView, ResponsePart},
};

use super::{ResolverResult, extension::SelectionSetResolverExtension, graphql::GraphqlResolver};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct LookupResolver {
    batch: bool,
    pub proxied: LookupProxiedResolver,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum LookupProxiedResolver {
    Graphql(GraphqlResolver),
    SelectionSetResolverExtension(SelectionSetResolverExtension),
}

impl LookupResolver {
    pub(in crate::resolver) async fn prepare(
        ctx: &PrepareContext<'_, impl Runtime>,
        definition: LookupResolverDefinition<'_>,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> PlanResult<Self> {
        let proxied = match definition.resolver().variant() {
            ResolverDefinitionVariant::GraphqlRootField(definition) => {
                GraphqlResolver::prepare(ctx, definition, plan_query_partition.selection_set())
                    .map(LookupProxiedResolver::Graphql)
            }
            ResolverDefinitionVariant::SelectionSetResolverExtension(definition) => {
                SelectionSetResolverExtension::prepare(ctx, definition, plan_query_partition.selection_set())
                    .await
                    .map(LookupProxiedResolver::SelectionSetResolverExtension)
            }
            ResolverDefinitionVariant::Lookup(_)
            | ResolverDefinitionVariant::Introspection(_)
            | ResolverDefinitionVariant::FieldResolverExtension(_)
            | ResolverDefinitionVariant::GraphqlFederationEntity(_) => {
                tracing::error!("Incompatible resolver for lookup.");
                Err(PlanError::Internal)
            }
        }?;
        Ok(LookupResolver {
            batch: definition.batch,
            proxied,
        })
    }

    pub(in crate::resolver) fn execute<'ctx, 'f, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        root_response_objects: ParentObjectsView<'_>,
        subgraph_response: ResponsePart<'ctx>,
    ) -> BoxFuture<'f, ResolverResult<'ctx, <R::Hooks as Hooks>::OnSubgraphResponseOutput>>
    where
        'ctx: 'f,
    {
        match &self.proxied {
            LookupProxiedResolver::Graphql(_) => unimplemented!("GB-8942"),
            LookupProxiedResolver::SelectionSetResolverExtension(resolver) => {
                let fut = resolver.execute_batch_lookup(ctx, plan, root_response_objects, subgraph_response);
                async move {
                    let result = fut.await;
                    ResolverResult {
                        execution: result,
                        on_subgraph_response_hook_output: None,
                    }
                }
                .boxed()
            }
        }
    }
}
