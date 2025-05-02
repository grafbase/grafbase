use std::borrow::Cow;

use futures::stream::BoxStream;
use futures_lite::{FutureExt, StreamExt};
use runtime::extension::{Data, FieldResolverExtension as _};
use walker::Walk;

use crate::{
    Runtime,
    execution::{ExecutionContext, ExecutionResult},
    prepare::{Plan, create_extension_directive_query_view},
    response::{GraphqlError, ResponseBuilder, ResponsePartBuilder},
};

use super::PreparedField;

impl super::FieldResolverExtension {
    pub(in crate::resolver) async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        new_response: impl Fn() -> ResponseBuilder<'ctx> + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<(ResponseBuilder<'ctx>, ResponsePartBuilder<'ctx>)>>> {
        let directive = self.directive_id.walk(ctx.schema());
        let subgraph_headers = ctx.subgraph_headers_with_rules(directive.subgraph().header_rules());

        let PreparedField {
            field_id,
            extension_data,
        } = self.prepared.first().unwrap();
        let field = plan.get_field(*field_id);
        let field_definition = field.definition();

        let query_view =
            create_extension_directive_query_view(ctx.schema(), directive, field.arguments(), ctx.variables());

        let stream = ctx
            .runtime()
            .extensions()
            .resolve_subscription_field(
                directive,
                field_definition,
                extension_data,
                subgraph_headers,
                query_view,
            )
            .boxed()
            .await
            .map_err(|err| err.with_location(field.location()))?;

        let stream = stream.map(move |result| {
            let mut response = new_response();
            let (root_object_id, part) = response.create_root_part();
            let response_part = part.into_shared();

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

            response_part
                .seed(plan.shape_id(), root_object_id)
                .deserialize_from_fields(&mut vec![(field, result)])
                .map_err(|err| {
                    tracing::error!("Failed to deserialize subgraph response: {}", err);
                    let field_id = self.prepared.first().unwrap().field_id;
                    let field = plan.get_field(field_id);

                    GraphqlError::invalid_subgraph_response().with_location(field.location())
                })?;

            Ok((response, response_part.unshare().unwrap()))
        });

        Ok(Box::pin(stream))
    }
}
