use std::borrow::Cow;

use bytes::Bytes;
use futures::{FutureExt, TryStreamExt};
use futures_util::{stream::BoxStream, StreamExt};
use headers::HeaderMapExt;
use runtime::fetch::{FetchRequest, Fetcher};
use serde::de::DeserializeSeed;
use url::Url;

use super::{
    deserialize::{GraphqlResponseSeed, RootGraphqlErrors},
    request::{retrying_fetch, SubgraphGraphqlRequest, SubgraphVariables},
    GraphqlResolver,
};
use crate::{
    execution::{ExecutionError, SubscriptionResponse},
    sources::{ExecutionResult, SubgraphRequestContext},
    Runtime,
};

impl GraphqlResolver {
    pub async fn execute_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &mut SubgraphRequestContext<'ctx, R>,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        if let Some(websocket_url) = ctx.endpoint().websocket_url() {
            self.execute_websocket_subscription(ctx, new_response, websocket_url)
                .await
        } else {
            self.execute_sse_subscription(ctx, new_response).await
        }
    }

    async fn execute_websocket_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &mut SubgraphRequestContext<'ctx, R>,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
        websocket_url: &'ctx Url,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        let endpoint = ctx.endpoint();

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

        let header_rules = ctx
            .execution_context()
            .subgraph_headers_with_rules(endpoint.header_rules());

        let headers = ctx
            .hooks()
            .on_subgraph_request(endpoint.subgraph_name(), http::Method::POST, &url, header_rules)
            .await?;

        let request = FetchRequest {
            url,
            method: http::Method::POST,
            headers,
            body: &SubgraphGraphqlRequest {
                query: &self.operation.query,
                variables: SubgraphVariables::<()> {
                    plan: ctx.plan(),
                    variables: &self.operation.variables,
                    extra_variables: Vec::new(),
                },
            },
            timeout: endpoint.timeout(),
        };

        let execution_ctx = ctx.execution_context();
        let fetch = move || {
            let result = execution_ctx
                .engine
                .runtime
                .fetcher()
                .graphql_over_websocket_stream(request.clone());

            result.then(|res| async { (res, None) })
        };

        let stream = retrying_fetch(ctx, fetch)
            .await?
            .map_err(move |error| ExecutionError::Fetch {
                subgraph_name: endpoint.subgraph_name().to_string(),
                error,
            })
            .map(move |subgraph_response| {
                let mut subscription_response = new_response();

                let resp = subscription_response.as_mut();
                GraphqlResponseSeed::new(
                    resp.next_seed(execution_ctx)
                        .expect("Must have a root object to update"),
                    RootGraphqlErrors::new(execution_ctx, resp),
                )
                .deserialize(subgraph_response?)?;

                Ok(subscription_response)
            });

        Ok(Box::pin(stream))
    }

    async fn execute_sse_subscription<'ctx, R: Runtime>(
        &'ctx self,
        ctx: &mut SubgraphRequestContext<'ctx, R>,
        new_response: impl Fn() -> SubscriptionResponse + Send + 'ctx,
    ) -> ExecutionResult<BoxStream<'ctx, ExecutionResult<SubscriptionResponse>>> {
        let endpoint = ctx.endpoint();

        let request = {
            let body = serde_json::to_vec(&SubgraphGraphqlRequest {
                query: &self.operation.query,
                variables: SubgraphVariables::<()> {
                    plan: ctx.plan(),
                    variables: &self.operation.variables,
                    extra_variables: Vec::new(),
                },
            })
            .map_err(|err| format!("Failed to serialize query: {err}"))?;

            let header_rules = ctx
                .execution_context()
                .subgraph_headers_with_rules(endpoint.header_rules());

            let mut headers = ctx
                .hooks()
                .on_subgraph_request(
                    endpoint.subgraph_name(),
                    http::Method::POST,
                    endpoint.url(),
                    header_rules,
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

        let execution_ctx = ctx.execution_context();
        let fetch = move || {
            let result = execution_ctx
                .engine
                .runtime
                .fetcher()
                .graphql_over_sse_stream(request.clone());

            result.then(|result| async { (result, None) })
        };

        let stream = retrying_fetch(ctx, fetch)
            .await?
            .map_err(move |error| ExecutionError::Fetch {
                subgraph_name: endpoint.subgraph_name().to_string(),
                error,
            })
            .map(move |subgraph_response| {
                let mut subscription_response = new_response();
                let resp = subscription_response.as_mut();

                GraphqlResponseSeed::new(
                    resp.next_seed(execution_ctx)
                        .expect("Must have a root object to update"),
                    RootGraphqlErrors::new(execution_ctx, resp),
                )
                .deserialize(&mut serde_json::Deserializer::from_slice(&subgraph_response?))?;

                Ok(subscription_response)
            });

        Ok(Box::pin(stream))
    }
}
