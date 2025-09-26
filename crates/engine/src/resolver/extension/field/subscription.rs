use std::borrow::Cow;

use futures::stream::BoxStream;
use futures_lite::{FutureExt, StreamExt};
use runtime::extension::{Data, FieldResolverExtension as _};
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{Plan, create_extension_directive_query_view},
    response::{ResponseBuilder, ResponsePartBuilder},
};

use super::PreparedField;

impl super::FieldResolverExtension {
    pub(in crate::resolver) async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        new_response: impl Fn() -> ResponseBuilder<'ctx> + Send + 'ctx,
    ) -> BoxStream<'ctx, (ResponseBuilder<'ctx>, ResponsePartBuilder<'ctx>)> {
        let directive = self.directive_id.walk(ctx.schema());
        let subgraph_headers = ctx.subgraph_headers_with_rules(
            directive
                .subgraph()
                .expect("Must be provided for resolvers")
                .header_rules(),
        );

        let PreparedField { field_id } = self.prepared.first().unwrap();
        let field = plan.get_field(*field_id);
        let field_definition = field.definition();

        let query_view =
            create_extension_directive_query_view(ctx.schema(), directive, field.arguments(), ctx.variables());

        let stream = match ctx
            .runtime()
            .extensions()
            .resolve_subscription_field(directive, field_definition, subgraph_headers, query_view)
            .boxed()
            .await
        {
            Ok(stream) => stream,
            Err(err) => {
                let mut response = new_response();
                let (_, mut part) = response.create_root_part();
                part.errors.push(err.with_location(field.location()));
                return Box::pin(futures_util::stream::once(std::future::ready((response, part))));
            }
        };

        let stream = stream.map(move |result| {
            tracing::debug!(
                "Received:\n{}",
                result
                    .as_ref()
                    .map(|data| match data {
                        Data::Json(bytes) => String::from_utf8_lossy(bytes),
                        Data::Cbor(bytes) => {
                            minicbor_serde::from_slice(bytes)
                                .ok()
                                .and_then(|v: sonic_rs::Value| sonic_rs::to_string_pretty(&v).ok().map(Into::into))
                                .unwrap_or_else(|| "<error>".into())
                        }
                    })
                    .unwrap_or(Cow::Borrowed("<error>"))
            );

            let mut response = new_response();
            let (parent_object, part) = response.create_root_part();
            let state = part.into_seed_state(plan.shape().id);
            state.ingest_fields(&parent_object, vec![(field, result)]);
            (response, state.into_response_part())
        });

        Box::pin(stream)
    }
}
