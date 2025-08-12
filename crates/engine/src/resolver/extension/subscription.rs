use futures::stream::BoxStream;
use futures_lite::{FutureExt, StreamExt};
use runtime::extension::ResolverExtension as _;
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
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());

        let prepared = self.prepared_fields.first().unwrap();

        let stream = ctx
            .runtime()
            .extensions()
            .resolve_subscription(
                EngineOperationContext::from(&ctx),
                definition.directive(),
                &prepared.extension_data,
                subgraph_headers,
                prepared.arguments.iter().map(|(id, argument_ids)| {
                    let arguments = argument_ids.walk(&ctx);
                    (*id, arguments.query_view(&schema::InputValueSet::All, ctx.variables()))
                }),
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
