use futures::{TryStreamExt, stream::BoxStream};
use futures_lite::{FutureExt, StreamExt};
use runtime::extension::{Data, ExtensionFieldDirective, ExtensionRuntime};
use walker::Walk;

use crate::{
    Runtime,
    execution::{ExecutionContext, ExecutionError, ExecutionResult, SubscriptionResponse},
    prepare::{Plan, create_extension_directive_query_view},
    response::GraphqlError,
};

use super::FieldResolverExtension;

impl FieldResolverExtension {
    pub(in crate::resolver) async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        let directive = self.directive_id.walk(ctx.schema());
        let headers = ctx.subgraph_headers_with_rules(directive.subgraph().header_rules());

        let field = plan.get_field(self.field_id);
        let field_definition = field.definition();

        let query_view =
            create_extension_directive_query_view(ctx.schema(), directive, field.arguments(), ctx.variables());

        let extension_directive = ExtensionFieldDirective {
            extension_id: directive.extension_id,
            subgraph: directive.subgraph(),
            field: field_definition,
            name: directive.name(),
            arguments: query_view,
        };

        let stream = ctx
            .engine
            .runtime
            .extensions()
            .resolve_subscription(headers, extension_directive)
            .boxed()
            .await
            .map_err(|err| GraphqlError::from(err).with_location(field.location()))?;

        let stream = stream
            .map_err(move |error| ExecutionError::from(error.to_string()))
            .map(move |response| {
                let data = response?;
                let mut subscription_response = new_response();

                let input_id = subscription_response.input_id();
                let response = subscription_response.as_mut();

                match &*data {
                    Data::JsonBytes(bytes) => {
                        tracing::debug!("Received:\n{}", String::from_utf8_lossy(bytes));

                        response
                            .seed(&ctx, input_id)
                            .deserialize_field_as_entity(
                                field.subgraph_response_key_str(),
                                &mut sonic_rs::Deserializer::from_slice(bytes),
                            )
                            .map_err(|err| {
                                tracing::error!("Failed to deserialize subgraph response: {}", err);
                                GraphqlError::invalid_subgraph_response().with_location(field.location())
                            })?;
                    }
                    Data::CborBytes(bytes) => {
                        tracing::debug!(
                            "Received:\n{}",
                            minicbor_serde::from_slice(bytes)
                                .ok()
                                .and_then(|v: sonic_rs::Value| sonic_rs::to_string_pretty(&v).ok())
                                .unwrap_or_else(|| "<error>".to_string())
                        );

                        response
                            .seed(&ctx, input_id)
                            .deserialize_field_as_entity(
                                field.subgraph_response_key_str(),
                                &mut minicbor_serde::Deserializer::new(bytes),
                            )
                            .map_err(|err| {
                                tracing::error!("Failed to deserialize subgraph response: {}", err);
                                GraphqlError::invalid_subgraph_response().with_location(field.location())
                            })?;
                    }
                }

                Ok(subscription_response)
            });

        Ok(Box::pin(stream))
    }
}
