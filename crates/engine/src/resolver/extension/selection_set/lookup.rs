use std::borrow::Cow;

use futures::{FutureExt as _, StreamExt, stream::FuturesUnordered};
use itertools::Itertools as _;
use runtime::extension::{Data, SelectionSetResolverExtension};
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::Plan,
    response::{ParentObjectsView, ResponsePartBuilder},
};

impl super::SelectionSetResolverExtension {
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

        let parent_objects = parent_objects_view.into_object_set();
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

            let state = response_part.into_seed_state(plan.shape().id);
            for (_, result) in results {
                match result {
                    Ok(data) => {
                        if let Err(Some(error)) =
                            state.deserialize_data_with(&data, state.parent_list_seed(&parent_objects))
                        {
                            state.insert_error_updates(&parent_objects, error);
                        }
                    }
                    Err(error) => state.insert_error_updates(&parent_objects, error),
                }
            }
            state.into_response_part()
        }
    }
}
