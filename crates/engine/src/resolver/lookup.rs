use futures::{FutureExt as _, future::BoxFuture};
use operation::{Operation, OperationContext};
use runtime::hooks::Hooks;
use schema::{LookupResolverDefinition, ResolverDefinitionVariant};

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{Plan, PlanError, PlanQueryPartition, PlanResult, PrepareContext},
    response::{ParentObjects, ResponsePartBuilder},
};

use super::{ResolverResult, extension::ExtensionResolver, graphql::GraphqlResolver};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct LookupResolver {
    batch: bool,
    pub proxied: LookupProxiedResolver,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum LookupProxiedResolver {
    Graphql(GraphqlResolver),
    Extension(ExtensionResolver),
}

impl LookupResolver {
    pub(in crate::resolver) async fn prepare(
        ctx: &PrepareContext<'_, impl Runtime>,
        operation: &Operation,
        definition: LookupResolverDefinition<'_>,
        plan_query_partition: PlanQueryPartition<'_>,
    ) -> PlanResult<Self> {
        let proxied = match definition.resolver().variant() {
            ResolverDefinitionVariant::GraphqlRootField(definition) => {
                let ctx = OperationContext {
                    schema: ctx.schema(),
                    operation,
                };
                GraphqlResolver::prepare(ctx, definition, plan_query_partition.selection_set())
                    .map(LookupProxiedResolver::Graphql)
            }
            ResolverDefinitionVariant::Extension(definition) => {
                ExtensionResolver::prepare(ctx, definition, plan_query_partition.selection_set())
                    .await
                    .map(LookupProxiedResolver::Extension)
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
        parent_objects_view: ParentObjects<'_>,
        subgraph_response: ResponsePartBuilder<'ctx>,
    ) -> BoxFuture<'f, ResolverResult<'ctx, <R::Hooks as Hooks>::OnSubgraphResponseOutput>>
    where
        'ctx: 'f,
    {
        match &self.proxied {
            LookupProxiedResolver::Graphql(_) => unimplemented!("GB-8942"),
            LookupProxiedResolver::Extension(resolver) => {
                let fut = resolver.execute_batch_lookup(ctx, plan, parent_objects_view, subgraph_response);
                async move {
                    let response_part = fut.await;
                    ResolverResult {
                        response_part,
                        on_subgraph_response_hook_output: None,
                    }
                }
                .boxed()
            }
        }
    }
}
