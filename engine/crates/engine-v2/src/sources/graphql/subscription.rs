use futures_util::{stream::BoxStream, StreamExt};
use runtime::{fetch::GraphqlRequest, rate_limiting::RateLimitKey};
use serde::de::DeserializeSeed;

use super::{
    deserialize::{GraphqlResponseSeed, RootGraphqlErrors},
    request::SubgraphVariables,
    GraphqlExecutor,
};
use crate::{
    execution::{ExecutionContext, ExecutionError, SubscriptionResponse},
    operation::PlanWalker,
    sources::ExecutionResult,
    Runtime,
};

impl GraphqlExecutor {
    pub async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx>,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        let endpoint = ctx.schema().walk(self.endpoint_id);

        let url = {
            let mut url = endpoint.websocket_url().clone();
            // If the user doesn't provide an explicit websocket URL we use the normal URL,
            // so make sure to convert the scheme to something appropriate
            match url.scheme() {
                "http" => url.set_scheme("ws").expect("this to work"),
                "https" => url.set_scheme("wss").expect("this to work"),
                _ => {}
            }
            url
        };

        ctx.engine
            .runtime
            .rate_limiter()
            .limit(&RateLimitKey::Subgraph(endpoint.subgraph_name().into()))
            .await?;

        let stream = ctx
            .engine
            .runtime
            .fetcher()
            .stream(GraphqlRequest {
                url: &url,
                query: &self.operation.query,
                variables: serde_json::to_value(&SubgraphVariables::<()> {
                    plan,
                    variables: &self.operation.variables,
                    extra_variables: Vec::new(),
                })
                .map_err(|error| error.to_string())?,
                headers: ctx.subgraph_headers_with_rules(endpoint.header_rules()),
            })
            .await
            .map_err(|error| ExecutionError::Fetch {
                subgraph_name: endpoint.subgraph_name().to_string(),
                error,
            })?;
        Ok(Box::pin(stream.map(move |subgraph_response| {
            let mut subscription_response = new_response();
            ingest_response(
                ctx,
                &mut subscription_response,
                subgraph_response.map_err(|error| ExecutionError::Fetch {
                    subgraph_name: endpoint.subgraph_name().to_string(),
                    error,
                })?,
            )?;
            Ok(subscription_response)
        })))
    }
}

fn ingest_response<R: Runtime>(
    ctx: ExecutionContext<'_, R>,
    subscription_response: &mut SubscriptionResponse,
    subgraph_response: serde_json::Value,
) -> ExecutionResult<()> {
    let response = subscription_response.root_response();
    GraphqlResponseSeed::new(
        response.next_seed(ctx).expect("Must have a root object to update"),
        RootGraphqlErrors::new(ctx, response),
    )
    .deserialize(subgraph_response)?;
    Ok(())
}
