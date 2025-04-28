use std::{borrow::Cow, sync::Arc};

use error::GraphqlError;
use futures::{FutureExt as _, StreamExt, stream::FuturesUnordered};
use itertools::Itertools as _;
use runtime::extension::{Data, SelectionSetResolverExtension};
use schema::InputValueSet;
use walker::Walk;

use crate::{
    Runtime,
    execution::{ExecutionContext, ExecutionResult},
    prepare::Plan,
    response::{InputResponseObjectSet, ResponseObjectsView, SubgraphResponse},
};

impl super::SelectionSetResolverExtension {
    pub(in crate::resolver) async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        input_object_refs: Arc<InputResponseObjectSet>,
        mut subgraph_response: SubgraphResponse,
    ) -> ExecutionResult<SubgraphResponse> {
        let definition = self.definition.walk(&ctx);
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());

        let mut results = self
            .prepared
            .iter()
            .map(|prepared| async {
                let field = plan.get_field(prepared.field_id);
                let result = ctx
                    .runtime()
                    .extensions()
                    .resolve_query_or_mutation_field(
                        definition.extension_id,
                        definition.subgraph().into(),
                        &prepared.extension_data,
                        // TODO: use Arc instead of clone?
                        subgraph_headers.clone(),
                        prepared.arguments.iter().map(|(id, argument_ids)| {
                            (
                                *id,
                                argument_ids.walk(&ctx).query_view(&InputValueSet::All, ctx.variables()),
                            )
                        }),
                    )
                    .boxed()
                    .await;
                (field, result)
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

        let input_object_id = input_object_refs.ids().next().ok_or("No object to update")?;

        tracing::debug!(
            "Received:\n{}",
            results
                .iter()
                .flat_map(|(field, result)| [
                    field.subgraph_response_key_str().into(),
                    match result {
                        Ok(Data::Json(bytes)) => String::from_utf8_lossy(bytes),
                        Ok(Data::Cbor(bytes)) => {
                            minicbor_serde::from_slice(bytes)
                                .ok()
                                .and_then(|v: sonic_rs::Value| sonic_rs::to_string_pretty(&v).ok().map(Into::into))
                                .unwrap_or_else(|| "<error>".into())
                        }
                        Err(_) => Cow::Borrowed("<error>"),
                    }
                ])
                .join("\n")
        );

        subgraph_response
            .as_shared_mut()
            .seed(&ctx, input_object_id)
            .deserialize_from_fields(&mut results)
            .map_err(|err| {
                tracing::error!("Failed to deserialize subgraph response: {}", err);
                let field_id = self.prepared.first().unwrap().field_id;
                let field = plan.get_field(field_id);

                GraphqlError::invalid_subgraph_response()
                    .with_location(field.location())
                    .with_path(&input_object_refs[input_object_id].path)
            })?;

        Ok(subgraph_response)
    }

    pub(in crate::resolver) fn execute_batch_lookup<'ctx, 'f, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        root_response_objects: ResponseObjectsView<'_>,
        mut subgraph_response: SubgraphResponse,
    ) -> impl Future<Output = ExecutionResult<SubgraphResponse>> + Send + 'f
    where
        'ctx: 'f,
    {
        let definition = self.definition.walk(&ctx);
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());

        let futures = self
            .prepared
            .iter()
            .map(|prepared| {
                let field = plan.get_field(prepared.field_id);
                ctx.runtime()
                    .extensions()
                    .resolve_query_or_mutation_field(
                        definition.extension_id,
                        definition.subgraph().into(),
                        &prepared.extension_data,
                        // TODO: use Arc instead of clone?
                        subgraph_headers.clone(),
                        prepared.arguments.iter().map(|(id, argument_ids)| {
                            (
                                *id,
                                argument_ids
                                    .walk(&ctx)
                                    .batch_view(ctx.variables(), root_response_objects.clone()),
                            )
                        }),
                    )
                    .boxed()
                    .map(move |result| (field, result))
            })
            .collect::<FuturesUnordered<_>>();

        let parent_objects = root_response_objects.into_input_object_refs();
        async move {
            let results = futures.collect::<Vec<_>>().await;

            tracing::debug!(
                "Received:\n{}",
                results
                    .iter()
                    .flat_map(|(field, result)| [
                        field.subgraph_response_key_str().into(),
                        match result {
                            Ok(Data::Json(bytes)) => String::from_utf8_lossy(bytes),
                            Ok(Data::Cbor(bytes)) => {
                                minicbor_serde::from_slice(bytes)
                                    .ok()
                                    .and_then(|v: sonic_rs::Value| sonic_rs::to_string_pretty(&v).ok().map(Into::into))
                                    .unwrap_or_else(|| "<error>".into())
                            }
                            Err(_) => Cow::Borrowed("<error>"),
                        }
                    ])
                    .join("\n")
            );

            let resp_mut = subgraph_response.as_shared_mut();

            for (_, result) in results {
                resp_mut.batch_seed(&ctx, parent_objects.clone()).ingest(result)
            }

            Ok(subgraph_response)
        }
    }
}
