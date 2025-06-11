use futures::{FutureExt as _, StreamExt, stream::FuturesUnordered};
use itertools::Itertools as _;
use runtime::extension::{Response, SelectionSetResolverExtension};
use schema::InputValueSet;
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::Plan,
    response::{ParentObjectSet, ResponsePartBuilder},
};

impl super::SelectionSetExtensionResolver {
    pub(in crate::resolver) async fn execute<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects: ParentObjectSet,
        response_part: ResponsePartBuilder<'ctx>,
    ) -> ResponsePartBuilder<'ctx> {
        let definition = self.definition.walk(&ctx);
        let subgraph_headers = ctx.subgraph_headers_with_rules(definition.subgraph().header_rules());

        let (parent_object_id, _) = parent_objects.iter_with_id().next().unwrap();
        let batched_field_results = self
            .prepared_fields
            .iter()
            .map(|prepared| async {
                let result = ctx
                    .runtime()
                    .extensions()
                    .resolve(
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
                (prepared.id, parent_object_id, Response::from(result))
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

        tracing::debug!(
            "Received:\n{}",
            batched_field_results
                .iter()
                .format_with("\n", |(field_id, parent_object_id, result), f| {
                    let field = plan.get_field(*field_id);
                    f(&format_args!(
                        "{} - {}\n{}",
                        field.subgraph_response_key_str(),
                        parent_object_id,
                        result
                    ))
                })
        );

        let state = response_part.into_seed_state(plan.shape().id);
        state.ingest_fields_host_batched(&parent_objects, self.prepared_fields.len(), batched_field_results);
        state.into_response_part()
    }
}
