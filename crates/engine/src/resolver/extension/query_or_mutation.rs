use futures::{FutureExt as _, StreamExt, future::BoxFuture, stream::FuturesUnordered};
use itertools::Itertools as _;
use runtime::extension::{EngineHooksExtension, ResolverExtension};
use walker::Walk;

use crate::{
    EngineOperationContext, Runtime,
    execution::ExecutionContext,
    prepare::Plan,
    response::{ParentObjects, ResponsePartBuilder},
};

impl super::ExtensionResolver {
    pub(in crate::resolver) fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: ParentObjects<'_>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> BoxFuture<'ctx, ResponsePartBuilder<'ctx>> {
        if self.definition.guest_batch {
            self.execute_guest_batch(ctx, plan, parent_objects, response_part)
        } else {
            self.execute_host_batch(ctx, plan, parent_objects, response_part)
        }
    }

    fn execute_guest_batch<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: ParentObjects<'_>,
        mut response_part: ResponsePartBuilder<'ctx>,
    ) -> BoxFuture<'ctx, ResponsePartBuilder<'ctx>> {
        let definition = self.definition.walk(&ctx);
        let headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());
        let extensions = ctx.runtime().extensions();
        let prepared_arguments = self
            .prepared_fields
            .iter()
            .map(|prepared| {
                let arguments = prepared.arguments.iter().map(|(id, argument_ids)| {
                    let arguments = argument_ids.walk(&ctx);
                    (*id, arguments.batch_view(ctx.variables(), &parent_objects))
                });
                (prepared, extensions.prepare_arguments(arguments))
            })
            .collect::<Vec<_>>();

        let parent_objects = parent_objects.into_object_set();
        Box::pin(async move {
            let headers = match extensions
                .on_virtual_subgraph_request(
                    EngineOperationContext::from(&ctx),
                    self.definition.subgraph_id.walk(&ctx),
                    headers,
                )
                .await
            {
                Ok(headers) => headers,
                Err(err) => {
                    tracing::error!("Error in on_virtual_subgraph_request: {}", err);
                    response_part.insert_error_updates(&parent_objects, plan.shape().id, [err]);
                    return response_part;
                }
            };

            let batched_field_results = prepared_arguments
                .into_iter()
                .map(|(prepared, arguments)| {
                    let field = plan.get_field(prepared.id);
                    extensions
                        .resolve(
                            EngineOperationContext::from(&ctx),
                            definition.directive(),
                            &prepared.extension_data,
                            headers.clone(),
                            arguments,
                        )
                        .boxed()
                        .map(move |result| (field, result))
                })
                .collect::<FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await;
            tracing::debug!(
                "Received:\n{}",
                batched_field_results
                    .iter()
                    .format_with("\n", |(field, result), f| {
                        f(&format_args!("{}: {}", field.subgraph_response_key_str(), result))
                    })
                    .to_string() // opentelemetry fails otherwise.
            );

            let state = response_part.into_seed_state(plan.shape().id);
            state.ingest_fields_guest_batched(&parent_objects, batched_field_results);
            state.into_response_part()
        })
    }

    fn execute_host_batch<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: ParentObjects<'_>,
        mut response_part: ResponsePartBuilder<'ctx>,
    ) -> BoxFuture<'ctx, ResponsePartBuilder<'ctx>> {
        let definition = self.definition.walk(&ctx);
        let headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());
        let extensions = ctx.runtime().extensions();

        let mut prepared_arguments = Vec::with_capacity(self.prepared_fields.len() * parent_objects.len());
        for prepared in &self.prepared_fields {
            for (parent_object_id, parent_object_view) in parent_objects.iter_with_id() {
                let arguments = extensions.prepare_arguments(prepared.arguments.iter().map(|(id, argument_ids)| {
                    let arguments = argument_ids.walk(&ctx);
                    (*id, arguments.view(ctx.variables(), parent_object_view))
                }));

                prepared_arguments.push((prepared, parent_object_id, arguments));
            }
        }

        let parent_objects = parent_objects.into_object_set();
        Box::pin(async move {
            let headers = match extensions
                .on_virtual_subgraph_request(
                    EngineOperationContext::from(&ctx),
                    self.definition.subgraph_id.walk(&ctx),
                    headers,
                )
                .await
            {
                Ok(headers) => headers,
                Err(err) => {
                    tracing::error!("Error in on_virtual_subgraph_request: {}", err);
                    response_part.insert_error_updates(&parent_objects, plan.shape().id, [err]);
                    return response_part;
                }
            };

            let batched_field_results = prepared_arguments
                .into_iter()
                .map(|(prepared, parent_object_id, arguments)| {
                    extensions
                        .resolve(
                            EngineOperationContext::from(&ctx),
                            definition.directive(),
                            &prepared.extension_data,
                            // TODO: use Arc instead of clone?
                            headers.clone(),
                            arguments,
                        )
                        .boxed()
                        .map(move |result| (prepared.id, parent_object_id, result))
                })
                .collect::<FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await;

            tracing::debug!(
                "Received:\n{}",
                batched_field_results
                    .iter()
                    .format_with("\n", |(field_id, parent_object_id, result), f| {
                        let field = plan.get_field(*field_id);
                        f(&format_args!(
                            "{} - {}\n{}",
                            field.subgraph_response_key_str(),
                            parent_object_id,
                            result
                        ))
                    })
                    .to_string() // opentelemetry fails otherwise.
            );

            let state = response_part.into_seed_state(plan.shape().id);
            state.ingest_fields_host_batched(&parent_objects, self.prepared_fields.len(), batched_field_results);
            state.into_response_part()
        })
    }
}
