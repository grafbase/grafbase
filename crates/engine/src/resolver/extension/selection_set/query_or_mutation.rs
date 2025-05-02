use std::{borrow::Cow, sync::Arc};

use error::GraphqlError;
use futures::{FutureExt as _, StreamExt, stream::FuturesUnordered};
use itertools::Itertools as _;
use runtime::extension::{Data, SelectionSetResolverExtension};
use schema::InputValueSet;
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::Plan,
    response::{ParentObjects, ParentObjectsView, ResponsePartBuilder},
};

impl super::SelectionSetResolverExtension {
    pub(in crate::resolver) async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: Arc<ParentObjects>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> ResponsePartBuilder<'ctx> {
        debug_assert!(parent_objects.len() == 1);
        let Some(root_object_id) = parent_objects.ids().next() else {
            return response_part;
        };

        let definition = self.definition.walk(&ctx);
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());

        let mut results = self
            .prepared_fields
            .iter()
            .map(|prepared| async {
                let field = plan.get_field(prepared.id);
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

        let part = response_part.into_shared();
        if let Err(err) = part
            .seed(plan.shape_id(), root_object_id)
            .deserialize_from_fields(&mut results)
        {
            tracing::error!("Failed to deserialize subgraph response: {}", err);
            part.borrow_mut()
                .insert_subgraph_failure(plan.shape_id(), GraphqlError::invalid_subgraph_response());
        }

        part.unshare().unwrap()
    }

    pub(in crate::resolver) fn execute_batch_lookup<'ctx, 'f, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects_view: ParentObjectsView<'_>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> impl Future<Output = ResponsePartBuilder<'ctx>> + Send + 'f
    where
        'ctx: 'f,
    {
        let definition = self.definition.walk(&ctx);
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());

        let futures = self
            .prepared_fields
            .iter()
            .map(|prepared| {
                let field = plan.get_field(prepared.id);
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
                                    .batch_view(ctx.variables(), parent_objects_view.clone()),
                            )
                        }),
                    )
                    .boxed()
                    .map(move |result| (field, result))
            })
            .collect::<FuturesUnordered<_>>();

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

            let part = response_part.into_shared();

            let shape_id = plan.shape_id_without_lookup_fields().unwrap();
            for (_, result) in results {
                part.batch_seed(shape_id).ingest(result)
            }

            part.unshare().unwrap()
        }
    }
}
