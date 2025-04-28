use std::sync::Arc;

use futures::{StreamExt as _, future::BoxFuture, stream::FuturesUnordered};
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

        let (fields, futures) = self
            .prepared
            .iter()
            .map(|prepared| {
                let field = plan.get_field(prepared.field_id);
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
                        &prepared.extension_data,
                        // TODO: use Arc instead of clone?
                        subgraph_headers.clone(),
                        query_view,
                        response_view.iter(),
                    )
                    .boxed();

                (field, future)
            })
            .unzip();

        let input_object_refs = root_response_objects.into_input_object_refs();

        Executor {
            subgraph_response,
            input_object_refs,
            fields,
            futures,
        }
    }
}

pub(in crate::resolver) struct Executor<'ctx> {
    subgraph_response: SubgraphResponse,
    input_object_refs: Arc<InputResponseObjectSet>,
    fields: Vec<SubgraphField<'ctx>>,
    #[allow(clippy::type_complexity)] // should be better with resolver rework... hopefully.
    futures: Vec<BoxFuture<'ctx, Result<Vec<Result<Data, GraphqlError>>, GraphqlError>>>,
}

impl<'ctx> Executor<'ctx> {
    pub async fn execute<R: Runtime>(self, ctx: ExecutionContext<'ctx, R>) -> ExecutionResult<SubgraphResponse> {
        let Self {
            mut subgraph_response,
            input_object_refs,
            fields,
            futures,
        } = self;

        let results = futures
            .into_iter()
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

        let mut field_results = fields
            .iter()
            .zip(results)
            .map(|(field, result)| match result {
                Ok(result) => (*field, result.into_iter()),
                Err(err) => (*field, vec![Err(err); input_object_refs.len()].into_iter()),
            })
            .collect::<Vec<_>>();

        let mut entity_fields = Vec::with_capacity(field_results.len());
        let response = subgraph_response.as_shared_mut();
        for input_object_id in input_object_refs.ids() {
            entity_fields.clear();
            for (field, results) in &mut field_results {
                entity_fields.push((*field, results.next().unwrap()));
            }

            response
                .seed(&ctx, input_object_id)
                .deserialize_from_fields(&mut entity_fields)
                .map_err(|err| {
                    tracing::error!("Failed to deserialize subgraph response: {}", err);

                    GraphqlError::invalid_subgraph_response()
                        .with_location(fields[0].location())
                        .with_path(&input_object_refs[input_object_id].path)
                })?;
        }

        Ok(subgraph_response)
    }
}
