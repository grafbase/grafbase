use futures::{TryStreamExt, future::BoxFuture, stream::BoxStream};
use futures_lite::StreamExt;
use runtime::{error::PartialGraphqlError, extension::Data};

use crate::{
    Runtime,
    execution::{ExecutionContext, ExecutionError, ExecutionResult, SubscriptionResponse},
    prepare::SubgraphField,
    response::GraphqlError,
};

pub(in crate::resolver) struct SubscriptionResolverExtensionRequest<'ctx> {
    pub(super) field: SubgraphField<'ctx>,
    pub(super) future: BoxFuture<'ctx, Result<BoxStream<'ctx, Result<Data, PartialGraphqlError>>, PartialGraphqlError>>,
}

impl<'ctx> SubscriptionResolverExtensionRequest<'ctx> {
    pub async fn execute_subscription<R: Runtime>(
        self,
        ctx: ExecutionContext<'ctx, R>,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        let Self { field, future } = self;

        let stream = match future.await {
            Ok(stream) => stream,
            Err(err) => return Err(ExecutionError::Graphql(err.into())),
        };

        let stream = stream
            .map_err(move |error| ExecutionError::from(error.to_string()))
            .map(move |response| {
                let data = response?;
                let mut subscription_response = new_response();

                let input_id = subscription_response.input_id();
                let response = subscription_response.as_mut();

                match data {
                    Data::JsonBytes(bytes) => {
                        tracing::debug!("Received:\n{}", String::from_utf8_lossy(&bytes));

                        response
                            .seed(&ctx, input_id)
                            .deserialize_field_as_entity(
                                field.subgraph_response_key_str(),
                                &mut sonic_rs::Deserializer::from_slice(&bytes),
                            )
                            .map_err(|err| {
                                tracing::error!("Failed to deserialize subgraph response: {}", err);
                                GraphqlError::invalid_subgraph_response()
                            })?;
                    }
                    Data::CborBytes(bytes) => {
                        tracing::debug!(
                            "Received:\n{}",
                            minicbor_serde::from_slice(&bytes)
                                .ok()
                                .and_then(|v: sonic_rs::Value| sonic_rs::to_string_pretty(&v).ok())
                                .unwrap_or_else(|| "<error>".to_string())
                        );

                        response
                            .seed(&ctx, input_id)
                            .deserialize_field_as_entity(
                                field.subgraph_response_key_str(),
                                &mut minicbor_serde::Deserializer::new(&bytes),
                            )
                            .map_err(|err| {
                                tracing::error!("Failed to deserialize subgraph response: {}", err);
                                GraphqlError::invalid_subgraph_response()
                            })?;
                    }
                }

                Ok(subscription_response)
            });

        Ok(Box::pin(stream))
    }
}
