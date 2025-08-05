mod nested;

use futures::{FutureExt as _, future::BoxFuture};
use operation::{Operation, OperationContext};
use schema::{LookupResolverDefinition, ResolverDefinitionVariant, StringId};
use walker::Walk as _;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{Plan, PlanError, PlanQueryPartition, PlanResult, PrepareContext},
    resolver::extension::SelectionSetExtensionResolver,
    response::{ParentObjects, ResponsePartBuilder},
};

use super::{ResolverResult, extension::ExtensionResolver, graphql::GraphqlResolver};

pub(in crate::resolver) use nested::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct LookupResolver {
    guest_batch: bool,
    namespace_key_id: Option<StringId>,
    pub proxied: LookupProxiedResolver,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum LookupProxiedResolver {
    Graphql(GraphqlResolver),
    Extension(ExtensionResolver),
    SelectionSetExtension(SelectionSetExtensionResolver),
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
            ResolverDefinitionVariant::SelectionSetResolverExtension(definition) => {
                SelectionSetExtensionResolver::prepare(ctx, definition, plan_query_partition.selection_set())
                    .await
                    .map(LookupProxiedResolver::SelectionSetExtension)
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
            guest_batch: definition.guest_batch,
            namespace_key_id: definition.namespace_key_id,
            proxied,
        })
    }

    pub(in crate::resolver) fn execute<'ctx, 'f, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: ParentObjects<'_>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> BoxFuture<'f, ResolverResult<'ctx>>
    where
        'ctx: 'f,
    {
        let namespace_key = self.namespace_key_id.walk(&ctx);
        if self.guest_batch {
            match &self.proxied {
                LookupProxiedResolver::Graphql(_) => unimplemented!("GB-8942"),
                LookupProxiedResolver::Extension(resolver) => {
                    let fut =
                        resolver.execute_guest_batch_lookup(ctx, plan, namespace_key, parent_objects, response_part);
                    async move {
                        let response_part = fut.await;
                        ResolverResult { response_part }
                    }
                    .boxed()
                }
                LookupProxiedResolver::SelectionSetExtension(resolver) => {
                    let fut = resolver.execute_batch_lookup(ctx, plan, parent_objects, response_part);
                    async move {
                        let response_part = fut.await;
                        ResolverResult { response_part }
                    }
                    .boxed()
                }
            }
        } else {
            match &self.proxied {
                LookupProxiedResolver::Graphql(_) => unimplemented!("GB-8942"),
                LookupProxiedResolver::Extension(resolver) => {
                    let fut =
                        resolver.execute_host_batch_lookup(ctx, plan, namespace_key, parent_objects, response_part);
                    async move {
                        let response_part = fut.await;
                        ResolverResult { response_part }
                    }
                    .boxed()
                }
                LookupProxiedResolver::SelectionSetExtension(_) => {
                    unimplemented!("Please update the extension to the latest Grafbase SDK.")
                }
            }
        }
    }
}
