use futures_util::{stream::BoxStream, StreamExt};
use runtime::fetch::GraphqlRequest;
use schema::sources::federation::{RootFieldResolverWalker, SubgraphHeaderValueRef, SubgraphWalker};
use url::Url;

use super::{
    deserialize::deserialize_response_into_output,
    query::{self, Query},
    ExecutionContext,
};
use crate::{
    plan::{PlanBoundary, PlanOutput},
    response::{ExecutorOutput, ResponseBuilder},
    sources::{ExecutorError, ExecutorResult, SubscriptionExecutor, SubscriptionResolverInput},
};

pub struct GraphqlSubscriptionExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    subgraph: SubgraphWalker<'ctx>,
    query: Query<'ctx>,
    plan_output: PlanOutput,
    plan_boundaries: Vec<PlanBoundary>,
}

impl<'ctx> GraphqlSubscriptionExecutor<'ctx> {
    pub fn build(
        resolver: RootFieldResolverWalker<'ctx>,
        SubscriptionResolverInput {
            ctx,
            plan_id,
            plan_output,
            plan_boundaries,
        }: SubscriptionResolverInput<'ctx>,
    ) -> ExecutorResult<SubscriptionExecutor<'ctx>> {
        let subgraph = resolver.subgraph();

        let query = query::Query::build(ctx, plan_id, &plan_output)
            .map_err(|err| ExecutorError::Internal(format!("Failed to build query: {err}")))?;

        Ok(SubscriptionExecutor::Graphql(Self {
            ctx,
            subgraph,
            query,
            plan_output,
            plan_boundaries,
        }))
    }

    pub async fn execute(self) -> ExecutorResult<BoxStream<'ctx, (ResponseBuilder, ExecutorOutput)>> {
        let Self {
            ctx,
            subgraph,
            query,
            plan_output,
            plan_boundaries,
        } = self;

        let url = {
            // This whole section is a hack because I've not done config for subscriptions yet.
            // We need a different URL for websockets vs normal HTTP calls.
            // For now we're just figuring out the URL based on what I've done in tests,
            // when we add config we can use the actual URL users provide.
            let mut url = Url::parse(subgraph.url()).expect("This is a temporary hack");
            url.set_scheme("ws").expect("this to work");
            url.set_path("ws");
            url.to_string()
        };

        let stream = ctx
            .engine
            .env
            .fetcher
            .stream(GraphqlRequest {
                url: &url,
                query: query.query,
                variables: serde_json::to_value(&query.variables)
                    .map_err(|error| ExecutorError::Internal(error.to_string()))?,
                headers: subgraph
                    .headers()
                    .filter_map(|header| {
                        Some((
                            header.name(),
                            match header.value() {
                                SubgraphHeaderValueRef::Forward(name) => self.ctx.header(name)?,
                                SubgraphHeaderValueRef::Static(value) => value,
                            },
                        ))
                    })
                    .collect(),
            })
            .await?;

        Ok(Box::pin(
            stream
                .take_while(|result| std::future::ready(result.is_ok()))
                .map(move |response| {
                    handle_response(
                        ctx,
                        &plan_output,
                        plan_boundaries.clone(),
                        response.expect("errors to be filtered out by the above take_while"),
                    )
                }),
        ))
    }
}

fn handle_response(
    ctx: ExecutionContext<'_>,
    plan_output: &PlanOutput,
    boundaries: Vec<PlanBoundary>,
    subgraph_response: serde_json::Value,
) -> (ResponseBuilder, ExecutorOutput) {
    let mut response = ResponseBuilder::new(ctx.walker.root_object_id());
    let mut output = response.new_output(boundaries);

    let boundary_item = response
        .root_response_boundary()
        .expect("a fresh response should always have a root");

    let err_path = boundary_item
        .response_path
        .child(ctx.walker.walk(plan_output.root_fields[0]).bound_response_key());

    let seed_ctx = ctx.seed_ctx(&mut output, plan_output);
    deserialize_response_into_output(
        &seed_ctx,
        &err_path,
        seed_ctx.create_root_seed(&boundary_item),
        subgraph_response,
    );

    (response, output)
}
