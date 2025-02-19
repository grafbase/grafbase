use std::sync::Arc;

use futures::future::BoxFuture;
use futures_lite::FutureExt;
use runtime::{
    error::PartialGraphqlError,
    extension::{Data, ExtensionFieldDirective, ExtensionRuntime},
};
use schema::{FieldResolverExtensionDefinition, FieldResolverExtensionDefinitionRecord};
use walker::Walk;

use crate::{
    execution::{ExecutionContext, ExecutionResult},
    prepare::{Plan, SubgraphField},
    resolver::Resolver,
    response::{GraphqlError, InputResponseObjectSet, ResponseObjectsView, SubgraphResponse},
    Runtime,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct FieldResolverExtension {
    pub definition: FieldResolverExtensionDefinitionRecord,
}

impl FieldResolverExtension {
    pub(in crate::resolver) fn prepare(definition: FieldResolverExtensionDefinition<'_>) -> Resolver {
        Resolver::FieldResolverExtension(Self {
            definition: *definition,
        })
    }

    pub(in crate::resolver) fn prepare_request<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        root_response_objects: ResponseObjectsView<'_>,
        subgraph_response: SubgraphResponse,
    ) -> FieldResolverExtensionRequest<'ctx> {
        let directive = self.definition.walk(ctx.schema()).directive();
        let field = plan
            .selection_set()
            .fields()
            .next()
            .expect("At least one field must be present");

        let field_definition = field.definition();
        let extension_directive = ExtensionFieldDirective {
            extension_id: directive.extension_id,
            subgraph: directive.subgraph(),
            field: field_definition,
            name: directive.name(),
            arguments: field
                .arguments()
                .into_extension_directive_query_view(directive, ctx.variables()),
        };

        let future = ctx
            .engine
            .runtime
            .extensions()
            .resolve_field(ctx.hooks_context, extension_directive, root_response_objects.iter())
            .boxed();

        let input_object_refs = root_response_objects.into_input_object_refs();
        FieldResolverExtensionRequest {
            field,
            subgraph_response,
            input_object_refs,
            future,
        }
    }
}

pub(in crate::resolver) struct FieldResolverExtensionRequest<'ctx> {
    field: SubgraphField<'ctx>,
    subgraph_response: SubgraphResponse,
    input_object_refs: Arc<InputResponseObjectSet>,
    future: BoxFuture<'ctx, Result<Vec<Result<Data, PartialGraphqlError>>, PartialGraphqlError>>,
}

impl<'ctx> FieldResolverExtensionRequest<'ctx> {
    pub async fn execute<R: Runtime>(self, ctx: ExecutionContext<'ctx, R>) -> ExecutionResult<SubgraphResponse> {
        let Self {
            field,
            mut subgraph_response,
            input_object_refs,
            future,
        } = self;

        match future.await {
            Ok(result) => {
                let response = subgraph_response.as_shared_mut();
                for (id, result) in input_object_refs.ids().zip(result) {
                    match result {
                        Ok(data) => match data {
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
                        },
                        Err(err) => response.borrow_mut().insert_errors(err, [id]),
                    }
                }
            }
            Err(err) => subgraph_response.set_subgraph_errors(vec![err.into()]),
        }

        Ok(subgraph_response)
    }
}
