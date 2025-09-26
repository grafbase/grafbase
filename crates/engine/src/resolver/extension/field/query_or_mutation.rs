use futures::{StreamExt as _, future::BoxFuture, stream::FuturesUnordered};
use futures_lite::FutureExt;
use runtime::extension::{Data, FieldResolverExtension as _};
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{
        Plan, RootFieldsShapeId, SubgraphField, create_extension_directive_query_view,
        create_extension_directive_response_view,
    },
    response::{GraphqlError, ParentObjectSet, ParentObjects, ResponsePartBuilder},
};

impl super::FieldResolverExtension {
    pub(in crate::resolver) fn build_executor<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: Plan<'ctx>,
        parent_objects_view: ParentObjects<'_>,
        subgraph_response: ResponsePartBuilder<'ctx>,
    ) -> Executor<'ctx> {
        let directive = self.directive_id.walk(ctx.schema());
        let subgraph_headers = ctx.subgraph_headers_with_rules(
            directive
                .subgraph()
                .expect("Must be provided for resolvers")
                .header_rules(),
        );

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
                    &parent_objects_view,
                );

                let future = ctx
                    .runtime()
                    .extensions()
                    .resolve_field(
                        directive,
                        field_definition,
                        // TODO: use Arc instead of clone?
                        subgraph_headers.clone(),
                        query_view,
                        response_view.iter(),
                    )
                    .boxed();

                (field, future)
            })
            .unzip();

        let parent_objects = parent_objects_view.into_object_set();

        Executor {
            shape_id: plan.shape().id,
            response_part: subgraph_response,
            parent_objects,
            fields,
            futures,
        }
    }
}

pub(in crate::resolver) struct Executor<'ctx> {
    shape_id: RootFieldsShapeId,
    response_part: ResponsePartBuilder<'ctx>,
    parent_objects: ParentObjectSet,
    fields: Vec<SubgraphField<'ctx>>,
    #[allow(clippy::type_complexity)] // should be better with resolver rework... hopefully.
    futures: Vec<BoxFuture<'ctx, Result<Vec<Result<Data, GraphqlError>>, GraphqlError>>>,
}

impl<'ctx> Executor<'ctx> {
    pub async fn execute(self) -> ResponsePartBuilder<'ctx> {
        let Self {
            shape_id,
            response_part,
            parent_objects,
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
                Err(err) => (*field, vec![Err(err); parent_objects.len()].into_iter()),
            })
            .collect::<Vec<_>>();

        let mut entity_fields = Vec::with_capacity(field_results.len());
        let state = response_part.into_seed_state(shape_id);
        entity_fields.clear();
        for (field, results) in &mut field_results {
            entity_fields.push((*field, results.next().unwrap()));
        }

        state.ingest_fields(
            parent_objects.iter().next().expect("Have at least one parent object"),
            entity_fields,
        );
        state.into_response_part()
    }
}
