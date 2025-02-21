use std::sync::Arc;

use futures::future::BoxFuture;
use runtime::{error::PartialGraphqlError, extension::Data};

use crate::{
    Runtime,
    execution::{ExecutionContext, ExecutionResult},
    prepare::SubgraphField,
    response::{GraphqlError, InputResponseObjectSet, SubgraphResponse},
};

pub(in crate::resolver) struct FieldResolverExtensionRequest<'ctx> {
    pub(super) field: SubgraphField<'ctx>,
    pub(super) subgraph_response: SubgraphResponse,
    pub(super) input_object_refs: Arc<InputResponseObjectSet>,
    pub(super) future: BoxFuture<'ctx, Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError>>,
}

impl<'ctx> FieldResolverExtensionRequest<'ctx> {
    pub async fn execute<R: Runtime>(self, ctx: ExecutionContext<'ctx, R>) -> ExecutionResult<SubgraphResponse> {
        let Self {
            field,
            mut subgraph_response,
            input_object_refs,
            future,
        } = self;

        let result = match future.await {
            Ok(result) => result,
            Err(err) => {
                subgraph_response.set_subgraph_errors(vec![err.into()]);
                return Ok(subgraph_response);
            }
        };

        let response = subgraph_response.as_shared_mut();

        for (id, result) in input_object_refs.ids().zip(result) {
            let data = match result {
                Ok(data) => data,
                Err(err) => {
                    response.borrow_mut().insert_errors(err, [id]);
                    continue;
                }
            };

            match data {
                Data::JsonBytes(bytes) => {
                    tracing::debug!("Received:\n{}", String::from_utf8_lossy(&bytes));

                    response
                        .seed(&ctx, id)
                        .deserialize_field_as_entity(
                            field.subgraph_response_key_str(),
                            &mut serde_json::Deserializer::from_slice(&bytes),
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
                            .and_then(|v: serde_json::Value| serde_json::to_string_pretty(&v).ok())
                            .unwrap_or_else(|| "<error>".to_string())
                    );

                    response
                        .seed(&ctx, id)
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
        }

        Ok(subgraph_response)
    }
}
