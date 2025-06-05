use futures::FutureExt as _;
use runtime::extension::{ResolverExtension, Response};
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::Plan,
    response::{ParentObjects, ResponsePartBuilder},
};

impl super::ExtensionResolver {
    pub(in crate::resolver) fn execute_batch_lookup<'ctx, 'f, R: Runtime>(
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
}
