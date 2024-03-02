use futures_util::{stream::BoxStream, StreamExt};
use runtime::fetch::GraphqlRequest;
use schema::sources::federation::{SubgraphHeaderValueRef, SubgraphWalker};

use super::{
    deserialize::ingest_deserializer_into_response, query::PreparedGraphqlOperation, variables::OutboundVariables,
    ExecutionContext, GraphqlExecutionPlan,
};
use crate::{
    plan::PlanWalker,
    response::{ResponseBuilder, ResponsePart},
    sources::{ExecutionError, ExecutionResult, SubscriptionExecutor, SubscriptionInput},
};

pub(crate) struct GraphqlSubscriptionExecutor<'ctx> {
    ctx: ExecutionContext<'ctx>,
    subgraph: SubgraphWalker<'ctx>,
    operation: &'ctx PreparedGraphqlOperation,
    plan: PlanWalker<'ctx>,
}

impl GraphqlExecutionPlan {
    pub fn new_subscription_executor<'ctx>(
        &'ctx self,
        input: SubscriptionInput<'ctx>,
    ) -> ExecutionResult<SubscriptionExecutor<'ctx>> {
        let SubscriptionInput { ctx, plan } = input;
        let subgraph = plan.schema().walk(self.subgraph_id);
        Ok(SubscriptionExecutor::Graphql(GraphqlSubscriptionExecutor {
            ctx,
            subgraph,
            operation: &self.operation,
            plan,
        }))
    }
}

impl<'ctx> GraphqlSubscriptionExecutor<'ctx> {
    pub async fn execute(self) -> ExecutionResult<BoxStream<'ctx, (ResponseBuilder, ResponsePart)>> {
        let Self {
            ctx,
            subgraph,
            operation,
            plan,
        } = self;

        let url = {
            let mut url = subgraph.websocket_url().clone();
            // If the user doesn't provide an explicit websocket URL we use the normal URL,
            // so make sure to convert the scheme to something appropriate
            match url.scheme() {
                "http" => url.set_scheme("ws").expect("this to work"),
                "https" => url.set_scheme("wss").expect("this to work"),
                _ => {}
            }
            url
        };

        let stream = ctx
            .engine
            .env
            .fetcher
            .stream(GraphqlRequest {
                url: &url,
                query: &operation.query,
                variables: serde_json::to_value(&OutboundVariables::new(plan.variables().collect()))
                    .map_err(|error| ExecutionError::Internal(error.to_string()))?,
                headers: subgraph
                    .headers()
                    .filter_map(|header| {
                        Some((
                            header.name(),
                            match header.value() {
                                SubgraphHeaderValueRef::Forward(name) => {
                                    self.ctx.headers.get(name).and_then(|value| value.to_str().ok())?
                                }
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
                        plan,
                        response.expect("errors to be filtered out by the above take_while"),
                    )
                }),
        ))
    }
}

fn handle_response(
    _ctx: ExecutionContext<'_>,
    plan: PlanWalker<'_>,
    subgraph_response: serde_json::Value,
) -> (ResponseBuilder, ResponsePart) {
    let mut response = ResponseBuilder::new(plan.operation().as_ref().root_object_id);
    let mut response_part = response.new_part(plan.output().boundary_ids);

    let boundary_item = response
        .root_response_boundary_item()
        .expect("a fresh response should always have a root");

    let err_path = plan.root_error_path(&boundary_item.response_path);
    let seed_ctx = plan.new_seed(&mut response_part);
    ingest_deserializer_into_response(
        &seed_ctx,
        &err_path,
        seed_ctx.create_root_seed(&boundary_item),
        subgraph_response,
    );

    (response, response_part)
}
