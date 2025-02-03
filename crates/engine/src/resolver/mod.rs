use extension::FieldResolverExtension;
use futures::FutureExt;
use futures_util::stream::BoxStream;
use grafbase_telemetry::graphql::OperationType;
use runtime::hooks::Hooks;
use schema::ResolverDefinitionVariant;
use std::future::Future;

use crate::{
    execution::{ExecutionContext, ExecutionError, ExecutionResult, SubscriptionResponse},
    prepare::{Plan, PlanQueryPartition, PlanResult},
    response::{ResponseObjectsView, SubgraphResponse},
    Runtime,
};

use self::{
    graphql::{FederationEntityResolver, GraphqlResolver},
    introspection::IntrospectionResolver,
};

mod extension;
mod graphql;
mod introspection;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) enum Resolver {
    Graphql(GraphqlResolver),
    FederationEntity(FederationEntityResolver),
    Introspection(IntrospectionResolver),
    FieldResolverExtension(FieldResolverExtension),
}

impl Resolver {
    pub fn prepare(operation_type: OperationType, plan_query_partition: PlanQueryPartition<'_>) -> PlanResult<Self> {
        match plan_query_partition.resolver_definition().variant() {
            ResolverDefinitionVariant::Introspection(_) => Ok(Resolver::Introspection(IntrospectionResolver)),
            ResolverDefinitionVariant::GraphqlRootField(definition) => {
                GraphqlResolver::prepare(definition, operation_type, plan_query_partition)
            }
            ResolverDefinitionVariant::GraphqlFederationEntity(definition) => {
                FederationEntityResolver::prepare(definition, plan_query_partition)
            }
            ResolverDefinitionVariant::FieldResolverExtension(definition) => {
                Ok(FieldResolverExtension::prepare(definition))
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
        plan: Plan<'ctx>,
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
            Resolver::Graphql(prepared) => {
                let input_object_refs = root_response_objects.into_input_object_refs();

                async move {
                    let mut ctx = prepared.build_subgraph_context(ctx);
                    let subgraph_result = prepared.execute(&mut ctx, input_object_refs, subgraph_response).await;
                    ctx.finalize(subgraph_result).await
                }
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
            Resolver::Introspection(prepared) => {
                let input_object_refs = root_response_objects.into_input_object_refs();
                async move {
                    let result = prepared.execute(ctx, plan, input_object_refs, subgraph_response);

                    ResolverResult {
                        execution: result,
                        on_subgraph_response_hook_output: None,
                    }
                }
            }
            .boxed(),
            Resolver::FieldResolverExtension(prepared) => {
                let request = prepared.prepare_request(ctx, plan, root_response_objects, subgraph_response);
                async move {
                    ResolverResult {
                        execution: request.execute(ctx).await,
                        on_subgraph_response_hook_output: None,
                    }
                }
                .boxed()
            }
        }
    }

    pub async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        _plan: Plan<'ctx>,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        match self {
            Resolver::Graphql(prepared) => {
                // TODO: for now we do not finalize this, e.g. we do not call the subgraph response hook. We should figure
                // out later what kind of data that hook would contain.
                let mut ctx = prepared.build_subgraph_context(ctx);
                prepared.execute_subscription(&mut ctx, new_response).await
            }
            Resolver::Introspection(_) => Err(ExecutionError::Internal(
                "Subscriptions can't contain introspection".into(),
            )),
            Resolver::FederationEntity(_) => Err(ExecutionError::Internal(
                "Subscriptions can only be at the root of a query so can't contain federated entitites".into(),
            )),
            Resolver::FieldResolverExtension(_) => Err(ExecutionError::Internal(
                "Subscriptions cannot be used with a field resolver extension.".into(),
            )),
        }
    }
}
