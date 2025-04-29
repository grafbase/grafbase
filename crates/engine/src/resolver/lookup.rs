use futures::{FutureExt as _, future::BoxFuture};
use runtime::hooks::Hooks;
use schema::{LookupResolverDefinition, ResolverDefinitionVariant};

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{Plan, PlanError, PlanQueryPartition, PlanResult, PrepareContext},
    response::{ParentObjectsView, ResponsePartBuilder},
};

use super::{ResolverResult, extension::SelectionSetResolverExtension, graphql::GraphqlResolver};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct LookupResolver {
    batch: bool,
    prepared: IndirectResolver,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum IndirectResolver {
    Graphql(GraphqlResolver),
    SelectionSetResolverExtension(SelectionSetResolverExtension),
}

impl LookupResolver {
    pub(in crate::resolver) async fn prepare(
        ctx: &PrepareContext<'_, impl Runtime>,
        definition: LookupResolverDefinition<'_>,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> PlanResult<Self> {
        let prepared = match definition.resolver().variant() {
            ResolverDefinitionVariant::GraphqlRootField(definition) => {
                GraphqlResolver::prepare(ctx, definition, plan_query_partition.selection_set())
                    .map(IndirectResolver::Graphql)
            }
            ResolverDefinitionVariant::SelectionSetResolverExtension(definition) => {
                SelectionSetResolverExtension::prepare(ctx, definition, plan_query_partition.selection_set())
                    .await
                    .map(IndirectResolver::SelectionSetResolverExtension)
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
            prepared,
        })
    }

    pub(in crate::resolver) fn execute<'ctx, 'f, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects_view: ParentObjectsView<'_>,
        subgraph_response: ResponsePartBuilder<'ctx>,
    ) -> BoxFuture<'f, ResolverResult<'ctx, <R::Hooks as Hooks>::OnSubgraphResponseOutput>>
    where
        'ctx: 'f,
    {
        match &self.prepared {
            IndirectResolver::Graphql(_) => unimplemented!("GB-8942"),
            IndirectResolver::SelectionSetResolverExtension(resolver) => {
                let fut = resolver.execute_batch_lookup(ctx, plan, parent_objects_view, subgraph_response);
                async move {
                    let response_part = fut.await;
                    ResolverResult {
                        execution: Ok(response_part),
                        on_subgraph_response_hook_output: None,
                    }
                }
                .boxed()
            }
        }
    }
}
