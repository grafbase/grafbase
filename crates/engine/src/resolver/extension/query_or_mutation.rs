use std::borrow::Cow;

use futures::{FutureExt as _, StreamExt, future::BoxFuture, stream::FuturesUnordered};
use itertools::Itertools as _;
use runtime::extension::{Data, ResolverExtension};
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::Plan,
    response::{ParentObjects, ResponsePartBuilder},
};

impl super::ExtensionResolver {
    pub(in crate::resolver) fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: ParentObjects<'_>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> BoxFuture<'ctx, ResponsePartBuilder<'ctx>> {
        if self.definition.guest_batch {
            self.execute_guest_batch(ctx, plan, parent_objects, response_part)
        } else {
            self.execute_host_batch(ctx, plan, parent_objects, response_part)
        }
    }

    fn execute_guest_batch<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: ParentObjects<'_>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> BoxFuture<'ctx, ResponsePartBuilder<'ctx>> {
        let definition = self.definition.walk(&ctx);
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());

        let futures = self
            .prepared_fields
            .iter()
            .map(|prepared| {
                let field = plan.get_field(prepared.id);
                ctx.runtime()
                    .extensions()
                    .resolve(
                        definition.extension_id,
                        definition.subgraph().into(),
                        &prepared.extension_data,
                        // TODO: use Arc instead of clone?
                        subgraph_headers.clone(),
                        prepared.arguments.iter().map(|(id, argument_ids)| {
                            let arguments = argument_ids.walk(&ctx);
                            (*id, arguments.batch_view(ctx.variables(), &parent_objects))
                        }),
                    )
                    .boxed()
                    .map(move |result| (field, result))
            })
            .collect::<FuturesUnordered<_>>();

        let parent_objects = parent_objects.into_object_set();
        Box::pin(async move {
            let batched_field_results = futures.collect::<Vec<_>>().await;
            tracing::debug!(
                "Received:\n{}",
                batched_field_results
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

            let state = response_part.into_seed_state(plan.shape().id);
            state.ingest_fields_guest_batched(&parent_objects, batched_field_results);
            state.into_response_part()
        })
    }

    fn execute_host_batch<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: ParentObjects<'_>,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> BoxFuture<'ctx, ResponsePartBuilder<'ctx>> {
        let definition = self.definition.walk(&ctx);
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());

        let futures = FuturesUnordered::new();
        for prepared in &self.prepared_fields {
            for (parent_object_id, parent_object_view) in parent_objects.iter_with_id() {
                futures.push(
                    ctx.runtime()
                        .extensions()
                        .resolve(
                            definition.extension_id,
                            definition.subgraph().into(),
                            &prepared.extension_data,
                            // TODO: use Arc instead of clone?
                            subgraph_headers.clone(),
                            prepared.arguments.iter().map(|(id, argument_ids)| {
                                let arguments = argument_ids.walk(&ctx);
                                (*id, arguments.view(ctx.variables(), parent_object_view))
                            }),
                        )
                        .boxed()
                        .map(move |result| (prepared.id, parent_object_id, result)),
                )
            }
        }

        let parent_objects = parent_objects.into_object_set();
        Box::pin(async move {
            let batched_field_results = futures.collect::<Vec<_>>().await;
            tracing::debug!(
                "Received:\n{}",
                batched_field_results
                    .iter()
                    .flat_map(|(field_id, parent_object_id, result)| [
                        format!(
                            "{} - {}",
                            plan.get_field(*field_id).subgraph_response_key_str(),
                            usize::from(*parent_object_id)
                        )
                        .into(),
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

            let state = response_part.into_seed_state(plan.shape().id);
            state.ingest_fields_host_batched(&parent_objects, self.prepared_fields.len(), batched_field_results);
            state.into_response_part()
        })
    }
}
