use futures::{FutureExt as _, StreamExt as _, stream::FuturesUnordered};
use runtime::extension::{EngineHooksExtension as _, ResolverExtension, Response};
use walker::Walk;

use crate::{
    EngineOperationContext, Runtime,
    execution::ExecutionContext,
    prepare::Plan,
    resolver::lookup::NestedSeed,
    response::{ParentObjects, ResponsePartBuilder},
};

impl super::ExtensionResolver {
    pub(in crate::resolver) fn execute_guest_batch_lookup<'ctx, 'f, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        namespace_key: Option<&'ctx str>,
        parent_objects: ParentObjects<'_>,
        mut response_part: ResponsePartBuilder<'ctx>,
    ) -> impl Future<Output = ResponsePartBuilder<'ctx>> + Send + 'f
    where
        'ctx: 'f,
    {
        debug_assert!(
            self.prepared_fields.len() == 1,
            "Expected exactly one prepared field for a lookup"
        );

        let definition = self.definition.walk(&ctx);
        let headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());
        let prepared = self.prepared_fields.first().unwrap();
        let field = plan.get_field(prepared.id);
        let extensions = ctx.runtime().extensions();
        let prepared_arguments = extensions.prepare_arguments(prepared.arguments.iter().map(|(id, argument_ids)| {
            (
                *id,
                argument_ids.walk(&ctx).batch_view(ctx.variables(), &parent_objects),
            )
        }));

        let parent_objects = parent_objects.into_object_set();
        async move {
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
            let response = extensions
                .resolve(
                    EngineOperationContext::from(&ctx),
                    definition.directive(),
                    &prepared.extension_data,
                    headers,
                    prepared_arguments,
                )
                .boxed()
                .await;
            tracing::debug!("Received for '{}':\n{}", field.subgraph_response_key_str(), response);

            let state = response_part.into_seed_state(plan.shape().id);
            match response {
                Response {
                    data: Some(data),
                    mut errors,
                } => {
                    let result = match namespace_key {
                        Some(key) => state.deserialize_data_with(
                            &data,
                            NestedSeed {
                                key,
                                seed: state.parent_list_seed(&parent_objects),
                            },
                        ),
                        None => state.deserialize_data_with(&data, state.parent_list_seed(&parent_objects)),
                    };
                    if let Err(Some(error)) = result {
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
        namespace_key: Option<&'ctx str>,
        parent_objects: ParentObjects<'_>,
        mut response_part: ResponsePartBuilder<'ctx>,
    ) -> impl Future<Output = ResponsePartBuilder<'ctx>> + Send + 'f
    where
        'ctx: 'f,
    {
        debug_assert!(
            self.prepared_fields.len() == 1,
            "Expected exactly one prepared field for a lookup"
        );

        let definition = self.definition.walk(&ctx);
        let headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());
        let prepared = self.prepared_fields.first().unwrap();
        let extensions = ctx.runtime().extensions();
        let prepared_arguments = parent_objects
            .iter_with_id()
            .map(|(parent_object_id, parent_object_view)| {
                let arguments = prepared.arguments.iter().map(|(id, argument_ids)| {
                    let arguments = argument_ids.walk(&ctx);
                    (*id, arguments.view(ctx.variables(), parent_object_view))
                });
                (parent_object_id, extensions.prepare_arguments(arguments))
            })
            .collect::<Vec<_>>();

        let parent_objects = parent_objects.into_object_set();
        async move {
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

            let field = plan.get_field(prepared.id);
            let state = response_part.into_seed_state(plan.shape().id);
            let mut futures = prepared_arguments
                .into_iter()
                .map(|(parent_object_id, arguments)| {
                    extensions
                        .resolve(
                            EngineOperationContext::from(&ctx),
                            definition.directive(),
                            &prepared.extension_data,
                            headers.clone(),
                            arguments,
                        )
                        .boxed()
                        .map(move |result| (parent_object_id, result))
                })
                .collect::<FuturesUnordered<_>>();

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
                        let result = match namespace_key {
                            Some(key) => state.deserialize_data_with(
                                &data,
                                NestedSeed {
                                    key,
                                    seed: state.parent_seed(parent_object),
                                },
                            ),
                            None => state.deserialize_data_with(&data, state.parent_seed(parent_object)),
                        };
                        if let Err(Some(error)) = result {
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
