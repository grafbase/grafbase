use std::borrow::Cow;

use bytes::Bytes;
use futures::{Future, TryStreamExt};
use futures_util::{stream::BoxStream, StreamExt};
use headers::HeaderMapExt;
use runtime::{
    fetch::{FetchRequest, FetchResult, Fetcher},
    rate_limiting::RateLimitKey,
};
use schema::sources::graphql::GraphqlEndpointWalker;
use serde::de::DeserializeSeed;
use tower::retry::budget::Budget;
use url::Url;
use web_time::Duration;

use super::{
    deserialize::{GraphqlResponseSeed, RootGraphqlErrors},
    record,
    request::{SubgraphGraphqlRequest, SubgraphVariables},
    GraphqlResolver,
};
use crate::{
    execution::{ExecutionContext, ExecutionError, SubscriptionResponse},
    operation::PlanWalker,
    sources::ExecutionResult,
    Runtime,
};

impl GraphqlResolver {
    pub async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx>,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        let endpoint = ctx.schema().walk(self.endpoint_id);
        if let Some(websocket_url) = endpoint.websocket_url() {
            self.execute_websocket_subscription(ctx, plan, new_response, endpoint, websocket_url)
                .await
        } else {
            self.execute_sse_subscription(ctx, plan, new_response, endpoint).await
        }
    }

    async fn execute_websocket_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx>,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
        endpoint: GraphqlEndpointWalker<'ctx>,
        websocket_url: &'ctx Url,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        // If the user doesn't provide an explicit websocket URL we use the normal URL,
        // so make sure to convert the scheme to something appropriate
        let url = match websocket_url.scheme() {
            "http" => {
                let mut url = websocket_url.clone();
                url.set_scheme("ws").expect("this to work");
                Cow::Owned(url)
            }
            "https" => {
                let mut url = websocket_url.clone();
                url.set_scheme("wss").expect("this to work");
                Cow::Owned(url)
            }
            _ => Cow::Borrowed(websocket_url),
        };

        let headers = ctx
            .hooks()
            .on_subgraph_request(
                endpoint.subgraph_name(),
                http::Method::POST,
                &url,
                ctx.subgraph_headers_with_rules(endpoint.header_rules()),
            )
            .await?;

        let request = FetchRequest {
            url,
            method: http::Method::POST,
            headers,
            body: &SubgraphGraphqlRequest {
                query: &self.operation.query,
                variables: SubgraphVariables::<()> {
                    plan,
                    variables: &self.operation.variables,
                    extra_variables: Vec::new(),
                },
            },
            timeout: endpoint.timeout(),
        };
        let stream = retrying_fetch(
            ctx,
            endpoint,
            ctx.engine.get_retry_budget_for_non_mutation(endpoint.id()),
            move || {
                ctx.engine
                    .runtime
                    .fetcher()
                    .graphql_over_websocket_stream(request.clone())
            },
        )
        .await?
        .map_err(move |error| ExecutionError::Fetch {
            subgraph_name: endpoint.subgraph_name().to_string(),
            error,
        })
        .map(move |subgraph_response| {
            let mut subscription_response = new_response();

            let resp = subscription_response.as_mut();
            GraphqlResponseSeed::new(
                resp.next_seed(ctx).expect("Must have a root object to update"),
                RootGraphqlErrors::new(ctx, resp),
            )
            .deserialize(subgraph_response?)?;

            Ok(subscription_response)
        });

        Ok(Box::pin(stream))
    }

    async fn execute_sse_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: ExecutionContext<'ctx, R>,
        plan: PlanWalker<'ctx>,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
        endpoint: GraphqlEndpointWalker<'ctx>,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        let request = {
            let body = serde_json::to_vec(&SubgraphGraphqlRequest {
                query: &self.operation.query,
                variables: SubgraphVariables::<()> {
                    plan,
                    variables: &self.operation.variables,
                    extra_variables: Vec::new(),
                },
            })
            .map_err(|err| format!("Failed to serialize query: {err}"))?;

            let mut headers = ctx
                .hooks()
                .on_subgraph_request(
                    endpoint.subgraph_name(),
                    http::Method::POST,
                    endpoint.url(),
                    ctx.subgraph_headers_with_rules(endpoint.header_rules()),
                )
                .await?;
            headers.typed_insert(headers::ContentType::json());
            headers.typed_insert(headers::ContentLength(body.len() as u64));
            headers.insert(
                http::header::ACCEPT,
                http::HeaderValue::from_static("text/even-stream,application/json;q=0.9"),
            );
            FetchRequest {
                url: Cow::Borrowed(endpoint.url()),
                method: http::Method::POST,
                headers,
                body: Bytes::from(body),
                timeout: endpoint.timeout(),
            }
        };

        let stream = retrying_fetch(
            ctx,
            endpoint,
            ctx.engine.get_retry_budget_for_non_mutation(endpoint.id()),
            move || ctx.engine.runtime.fetcher().graphql_over_sse_stream(request.clone()),
        )
        .await?
        .map_err(move |error| ExecutionError::Fetch {
            subgraph_name: endpoint.subgraph_name().to_string(),
            error,
        })
        .map(move |subgraph_response| {
            let mut subscription_response = new_response();

            let resp = subscription_response.as_mut();
            GraphqlResponseSeed::new(
                resp.next_seed(ctx).expect("Must have a root object to update"),
                RootGraphqlErrors::new(ctx, resp),
            )
            .deserialize(&mut serde_json::Deserializer::from_slice(&subgraph_response?))?;

            Ok(subscription_response)
        });

        Ok(Box::pin(stream))
    }
}

async fn retrying_fetch<'ctx, R: Runtime, F, T>(
    ctx: ExecutionContext<'ctx, R>,
    endpoint: GraphqlEndpointWalker<'_>,
    retry_budget: Option<&Budget>,
    fetch: impl Fn() -> F + Send + Sync,
) -> ExecutionResult<T>
where
    F: Future<Output = FetchResult<T>> + Send,
    T: Send,
{
    let mut result = rate_limited_fetch(ctx, endpoint, &fetch).await;

    let Some(retry_budget) = retry_budget else {
        return result;
    };

    let mut counter = 0;

    loop {
        match result {
            Ok(bytes) => {
                retry_budget.deposit();
                return Ok(bytes);
            }
            Err(err) => {
                if retry_budget.withdraw().is_ok() {
                    let jitter = rand::random::<f64>() * 2.0;
                    let exp_backoff = (100 * 2u64.pow(counter)) as f64;
                    let backoff_ms = (exp_backoff * jitter).round() as u64;

                    ctx.engine.runtime.sleep(Duration::from_millis(backoff_ms)).await;
                    record::subgraph_retry(ctx, endpoint, false);

                    counter += 1;

                    result = rate_limited_fetch(ctx, endpoint, &fetch).await;
                } else {
                    record::subgraph_retry(ctx, endpoint, true);
                    return Err(err);
                }
            }
        }
    }
}

async fn rate_limited_fetch<'ctx, R: Runtime, F, T>(
    ctx: ExecutionContext<'ctx, R>,
    endpoint: GraphqlEndpointWalker<'ctx>,
    fetch: impl Fn() -> F + Send,
) -> ExecutionResult<T>
where
    F: Future<Output = FetchResult<T>> + Send,
    T: Send,
{
    ctx.engine
        .runtime
        .rate_limiter()
        .limit(&RateLimitKey::Subgraph(endpoint.subgraph_name().into()))
        .await?;

    record::increment_inflight_requests(ctx, endpoint);
    let result = fetch().await;
    record::decrement_inflight_requests(ctx, endpoint);

    result.map_err(|error| ExecutionError::Fetch {
        subgraph_name: endpoint.subgraph_name().to_string(),
        error,
    })
}
