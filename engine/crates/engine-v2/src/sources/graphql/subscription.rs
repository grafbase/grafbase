use futures_util::{stream::BoxStream, StreamExt};
use runtime::fetch::GraphqlRequest;
use schema::sources::graphql::GraphqlEndpointWalker;
use serde::de::DeserializeSeed;

use super::{
    deserialize::{GraphqlResponseSeed, RootGraphqlErrors},
    query::PreparedGraphqlOperation,
    variables::SubgraphVariables,
    ExecutionContext, GraphqlPreparedExecutor,
};
use crate::{
    execution::{PlanWalker, SubscriptionResponse},
    sources::{ExecutionResult, SubscriptionExecutor, SubscriptionInput},
    Runtime,
};

pub(crate) struct GraphqlSubscriptionExecutor<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    subgraph: GraphqlEndpointWalker<'ctx>,
    operation: &'ctx PreparedGraphqlOperation,
    plan: PlanWalker<'ctx>,
}

impl GraphqlPreparedExecutor {
    pub fn new_subscription_executor<'ctx, R: Runtime>(
        &'ctx self,
        input: SubscriptionInput<'ctx, R>,
    ) -> ExecutionResult<SubscriptionExecutor<'ctx, R>> {
        let SubscriptionInput { ctx, plan } = input;
        let subgraph = ctx.schema().walk(self.subgraph_id);
        Ok(SubscriptionExecutor::Graphql(GraphqlSubscriptionExecutor {
            ctx,
            subgraph,
            operation: &self.operation,
            plan,
        }))
    }
}

impl<'ctx, R: Runtime> GraphqlSubscriptionExecutor<'ctx, R> {
    pub async fn execute(
        self,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
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

        self.ctx
            .engine
            .runtime
            .rate_limiter()
            .limit(&crate::engine::RateLimitContext::Subgraph(self.subgraph.name()))
            .await?;

        let stream = ctx
            .engine
            .runtime
            .fetcher()
            .stream(GraphqlRequest {
                url: &url,
                query: &operation.query,
                variables: serde_json::to_value(&SubgraphVariables {
                    plan,
                    variables: &operation.variables,
                    inputs: Vec::new(),
                })
                .map_err(|error| error.to_string())?,
                headers: self.ctx.headers_with_rules(subgraph.header_rules()),
            })
            .await?;

        Ok(Box::pin(stream.map(move |subgraph_response| {
            let mut subscription_response = new_response();
            ingest_response(&mut subscription_response, plan, subgraph_response?)?;
            Ok(subscription_response)
        })))
    }
}

fn ingest_response(
    execution: &mut SubscriptionResponse,
    plan: PlanWalker<'_>,
    subgraph_response: serde_json::Value,
) -> ExecutionResult<()> {
    let part = execution.root_response_part().as_mut();
    GraphqlResponseSeed::new(
        part.next_seed(plan).expect("Must have a root object to update"),
        RootGraphqlErrors {
            response_part: &part,
            response_keys: plan.response_keys(),
        },
    )
    .deserialize(subgraph_response)?;
    Ok(())
}
