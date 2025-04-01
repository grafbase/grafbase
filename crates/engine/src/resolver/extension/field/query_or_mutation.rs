use std::sync::Arc;

use futures::future::BoxFuture;
use futures_lite::FutureExt;
use runtime::extension::{Data, FieldResolverExtension as _};
use walker::Walk;

use crate::{
    Runtime,
    execution::{ExecutionContext, ExecutionResult},
    prepare::{Plan, SubgraphField, create_extension_directive_query_view, create_extension_directive_response_view},
    response::{GraphqlError, InputResponseObjectSet, ResponseObjectsView, SubgraphResponse},
};

impl super::FieldResolverExtension {
    pub(in crate::resolver) fn build_executor<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        root_response_objects: ResponseObjectsView<'_>,
        subgraph_response: SubgraphResponse,
    ) -> Executor<'ctx> {
        let directive = self.directive_id.walk(ctx.schema());
        let subgraph_headers = ctx.subgraph_headers_with_rules(directive.subgraph().header_rules());

        let field = plan.get_field(self.field_id);
        let field_definition = field.definition();

        let query_view =
            create_extension_directive_query_view(ctx.schema(), directive, field.arguments(), ctx.variables());

        let response_view = create_extension_directive_response_view(
            ctx.schema(),
            directive,
            field.arguments(),
            ctx.variables(),
            root_response_objects.clone(),
        );

        let future = ctx
            .runtime()
            .extensions()
            .resolve_field(
                directive,
                field_definition,
                &self.prepared_data,
                subgraph_headers,
                query_view,
                response_view.iter(),
            )
            .boxed();

        let input_object_refs = root_response_objects.into_input_object_refs();

        Executor {
            field,
            subgraph_response,
            input_object_refs,
            future,
        }
    }
}

pub(in crate::resolver) struct Executor<'ctx> {
    pub(super) field: SubgraphField<'ctx>,
    pub(super) subgraph_response: SubgraphResponse,
    pub(super) input_object_refs: Arc<InputResponseObjectSet>,
    pub(super) future: BoxFuture<'ctx, Result<Vec<Result<Data, GraphqlError>>, GraphqlError>>,
}

impl<'ctx> Executor<'ctx> {
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
                subgraph_response.set_subgraph_errors(vec![err.with_location(field.location())]);
                return Ok(subgraph_response);
            }
        };

        let response = subgraph_response.as_shared_mut();

        for (id, result) in input_object_refs.ids().zip(result) {
            let data = match result {
                Ok(data) => data,
                Err(err) => {
                    response.borrow_mut().insert_errors(
                        err.with_location(field.location())
                            .with_path(&input_object_refs[id].path),
                        [id],
                    );
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
                            &mut sonic_rs::Deserializer::from_slice(&bytes),
                        )
                        .map_err(|err| {
                            tracing::error!("Failed to deserialize subgraph response: {}", err);
                            GraphqlError::invalid_subgraph_response()
                                .with_location(field.location())
                                .with_path(&input_object_refs[id].path)
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
                        .seed(&ctx, id)
                        .deserialize_field_as_entity(
                            field.subgraph_response_key_str(),
                            &mut minicbor_serde::Deserializer::new(&bytes),
                        )
                        .map_err(|err| {
                            tracing::error!("Failed to deserialize subgraph response: {}", err);
                            GraphqlError::invalid_subgraph_response()
                                .with_location(field.location())
                                .with_path(&input_object_refs[id].path)
                        })?;
                }
            }
        }

        Ok(subgraph_response)
    }
}
