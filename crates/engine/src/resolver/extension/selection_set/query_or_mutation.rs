use std::borrow::Cow;

use futures::{FutureExt as _, StreamExt, stream::FuturesUnordered};
use itertools::Itertools as _;
use runtime::extension::{Data, SelectionSetResolverExtension};
use schema::InputValueSet;
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::Plan,
    response::{ParentObjects, ResponsePartBuilder},
};

impl super::SelectionSetResolverExtension {
    pub(in crate::resolver) async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: ParentObjects,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> ResponsePartBuilder<'ctx> {
        let definition = self.definition.walk(&ctx);
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());

        let mut fields = self
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
            fields
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
        state.ingest_fields(
            parent_objects.iter().next().expect("Have at least one parent object"),
            &mut fields,
        );
        state.into_response_part()
    }
}
