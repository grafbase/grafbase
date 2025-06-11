use futures::FutureExt as _;
use runtime::extension::{Response, SelectionSetResolverExtension as _};
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::Plan,
    response::{ParentObjects, ResponsePartBuilder},
};

impl super::SelectionSetExtensionResolver {
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
        let definition = self.definition.walk(&ctx);
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());
        let prepared = self.prepared_fields.first().unwrap();
        let field = plan.get_field(prepared.id);

        let fut = ctx
            .runtime()
            .extensions()
            .resolve(
                definition.extension_id,
                definition.subgraph().into(),
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
            let result = fut.await;
            tracing::debug!(
                "Received for '{}':\n{}",
                field.subgraph_response_key_str(),
                Response::from(result.clone())
            );

            let state = response_part.into_seed_state(plan.shape().id);
            match result {
                Ok(data) => {
                    if let Err(Some(err)) = state.deserialize_data_with(&data, state.parent_list_seed(&parent_objects))
                    {
                        state.insert_errors(parent_objects.iter().next().unwrap(), [err]);
                    }
                }
                Err(err) => {
                    state.insert_error_updates(&parent_objects, [err]);
                }
            };

            state.into_response_part()
        }
    }
}
