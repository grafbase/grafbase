use futures_util::{stream::BoxStream, StreamExt};
use runtime::fetch::GraphqlRequest;
use serde::de::DeserializeSeed;

use super::{
    deserialize::{GraphqlResponseSeed, RootGraphqlErrors},
    variables::SubgraphVariables,
    ExecutionContext, GraphqlPreparedExecutor,
};
use crate::{
    execution::{PlanWalker, SubscriptionResponse},
    sources::ExecutionResult,
    Runtime,
};

impl GraphqlPreparedExecutor {
    pub async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx>,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        let subgraph = ctx.schema().walk(self.subgraph_id);

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
            .runtime
            .fetcher()
            .stream(GraphqlRequest {
                url: &url,
                query: &self.operation.query,
                variables: serde_json::to_value(&SubgraphVariables {
                    plan,
                    variables: &self.operation.variables,
                    inputs: Vec::new(),
                })
                .map_err(|error| error.to_string())?,
                headers: ctx.headers_with_rules(subgraph.header_rules()),
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
    subscription_response: &mut SubscriptionResponse,
    plan: PlanWalker<'_>,
    subgraph_response: serde_json::Value,
) -> ExecutionResult<()> {
    let response = subscription_response.root_response();
    GraphqlResponseSeed::new(
        response.next_seed(plan).expect("Must have a root object to update"),
        RootGraphqlErrors {
            response,
            response_keys: plan.response_keys(),
        },
    )
    .deserialize(subgraph_response)?;
    Ok(())
}
