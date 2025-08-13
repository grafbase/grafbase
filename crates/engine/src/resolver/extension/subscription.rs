use futures::stream::BoxStream;
use futures_lite::{FutureExt, StreamExt};
use runtime::extension::{EngineHooksExtension as _, ResolverExtension as _};
use walker::Walk;

use crate::{
    EngineOperationContext, Runtime,
    execution::ExecutionContext,
    prepare::Plan,
    response::{ResponseBuilder, ResponsePartBuilder},
};

impl super::ExtensionResolver {
    pub(in crate::resolver) async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        new_response: impl Fn() -> ResponseBuilder<'ctx> + Send + 'ctx,
    ) -> BoxStream<'ctx, (ResponseBuilder<'ctx>, ResponsePartBuilder<'ctx>)> {
        let definition = self.definition.walk(&ctx);
        let headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());
        let extensions = ctx.runtime().extensions();

        let prepared = self.prepared_fields.first().unwrap();
        let prepared_arguments = extensions.prepare_arguments(prepared.arguments.iter().map(|(id, argument_ids)| {
            let arguments = argument_ids.walk(&ctx);
            (*id, arguments.query_view(&schema::InputValueSet::All, ctx.variables()))
        }));

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
                let mut response = new_response();
                let (parent_object, mut part) = response.create_root_part();
                part.insert_error_update(&parent_object, plan.shape().id, [err]);
                return futures_util::stream::iter([(response, part)]).boxed();
            }
        };
        let stream = extensions
            .resolve_subscription(
                EngineOperationContext::from(&ctx),
                definition.directive(),
                &prepared.extension_data,
                headers,
                prepared_arguments,
            )
            .boxed()
            .await;

        Box::pin(stream.map(move |subgraph_response| {
            tracing::debug!("Received:\n{subgraph_response}");

            let mut response = new_response();
            let (parent_object, part) = response.create_root_part();
            let state = part.into_seed_state(plan.shape().id);
            state.ingest_subscription_field(&parent_object, prepared.id, subgraph_response);
            (response, state.into_response_part())
        }))
    }
}
