use futures::{FutureExt as _, StreamExt as _, stream::FuturesUnordered};
use runtime::extension::{ResolverExtension, Response};
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::Plan,
    response::{ParentObjects, ResponsePartBuilder},
};

impl super::ExtensionResolver {
    pub(in crate::resolver) fn execute_guest_batch_lookup<'ctx, 'f, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: ParentObjects<'_>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> impl Future<Output = ResponsePartBuilder<'ctx>> + Send + 'f
    where
        'ctx: 'f,
    {
        debug_assert!(
            self.prepared_fields.len() == 1,
            "Expected exactly one prepared field for a lookup"
        );

        let definition = self.definition.walk(&ctx);
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());
        let prepared = self.prepared_fields.first().unwrap();
        let field = plan.get_field(prepared.id);

        let fut = ctx
            .runtime()
            .extensions()
            .resolve(
                &ctx.request_context.extension_context,
                definition.directive(),
                &prepared.extension_data,
                // TODO: use Arc instead of clone?
                subgraph_headers.clone(),
                prepared.arguments.iter().map(|(id, argument_ids)| {
                    (
                        *id,
                        argument_ids.walk(&ctx).batch_view(ctx.variables(), &parent_objects),
                    )
                }),
            )
            .boxed();

        let parent_objects = parent_objects.into_object_set();
        async move {
            let response = fut.await;
            tracing::debug!("Received for '{}':\n{}", field.subgraph_response_key_str(), response);

            let state = response_part.into_seed_state(plan.shape().id);
            match response {
                Response {
                    data: Some(data),
                    mut errors,
                } => {
                    if let Err(Some(error)) =
                        state.deserialize_data_with(&data, state.parent_list_seed(&parent_objects))
                    {
                        errors.push(error);
                        state.insert_errors(parent_objects.iter().next().unwrap(), errors);
                    } else {
                        state.insert_errors(parent_objects.iter().next().unwrap(), errors);
                    }
                }
                Response { data: None, errors } => {
                    state.insert_error_updates(&parent_objects, errors);
                }
            };

            state.into_response_part()
        }
    }

    pub(in crate::resolver) fn execute_host_batch_lookup<'ctx, 'f, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: ParentObjects<'_>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> impl Future<Output = ResponsePartBuilder<'ctx>> + Send + 'f
    where
        'ctx: 'f,
    {
        debug_assert!(
            self.prepared_fields.len() == 1,
            "Expected exactly one prepared field for a lookup"
        );

        let definition = self.definition.walk(&ctx);
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());
        let prepared = self.prepared_fields.first().unwrap();

        let mut futures = FuturesUnordered::new();
        for (parent_object_id, parent_object_view) in parent_objects.iter_with_id() {
            futures.push(
                ctx.runtime()
                    .extensions()
                    .resolve(
                        &ctx.request_context.extension_context,
                        definition.directive(),
                        &prepared.extension_data,
                        // TODO: use Arc instead of clone?
                        subgraph_headers.clone(),
                        prepared.arguments.iter().map(|(id, argument_ids)| {
                            let arguments = argument_ids.walk(&ctx);
                            (*id, arguments.view(ctx.variables(), parent_object_view))
                        }),
                    )
                    .boxed()
                    .map(move |result| (parent_object_id, result)),
            );
        }

        let parent_objects = parent_objects.into_object_set();
        async move {
            let field = plan.get_field(prepared.id);
            let state = response_part.into_seed_state(plan.shape().id);
            while let Some((parent_object_id, response)) = futures.next().await {
                let parent_object = &parent_objects[parent_object_id];
                tracing::debug!(
                    "Received for {} - {}:\n{}",
                    field.subgraph_response_key_str(),
                    parent_object_id,
                    response
                );
                match response {
                    Response {
                        data: Some(data),
                        mut errors,
                    } => {
                        if let Err(Some(error)) = state.deserialize_data_with(&data, state.parent_seed(parent_object)) {
                            errors.push(error);
                            state.insert_errors(parent_object, errors);
                        } else {
                            state.insert_errors(parent_object, errors);
                        }
                    }
                    Response { data: None, errors } => {
                        state.insert_error_update(parent_object, errors);
                    }
                };
            }

            state.into_response_part()
        }
    }
}
