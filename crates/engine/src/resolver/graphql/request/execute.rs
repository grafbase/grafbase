use std::{borrow::Cow, time::Duration};

use bytes::Bytes;
use event_queue::RequestExecution;
use futures::Future;
use grafbase_telemetry::{
    graphql::GraphqlResponseStatus, otel::tracing_opentelemetry::OpenTelemetrySpanExt as _,
    span::subgraph::SubgraphHttpRequestSpan,
};
use headers::HeaderMapExt;
use runtime::{
    fetch::{FetchError, FetchRequest, FetchResult, Fetcher},
    rate_limiting::RateLimitKey,
};
use tower::retry::budget::Budget;
use tracing::{Instrument, Span};

use crate::{
    Runtime,
    execution::{ExecutionError, ExecutionResult},
    resolver::graphql::SubgraphContext,
    response::{ErrorCode, GraphqlError, ResponsePartBuilder},
};

pub trait ResponseIngester: Send {
    // Because of this https://github.com/rust-lang/rust/issues/110338#issuecomment-1513761297
    // We can't have ResponseIngester have a lifetime easily, so we pass the response_part as an
    // argument to circumvent the issue.
    fn ingest(
        self,
        result: Result<http::Response<Bytes>, GraphqlError>,
        response_part: ResponsePartBuilder<'_>,
    ) -> impl Future<Output = (Option<GraphqlResponseStatus>, ResponsePartBuilder<'_>)> + Send;
}

pub(crate) async fn execute_subgraph_request<'ctx, R: Runtime>(
    ctx: &mut SubgraphContext<'ctx, R>,
    mut headers: http::HeaderMap,
    body: impl Into<Bytes> + Send,
    response_part: ResponsePartBuilder<'ctx>,
    ingester: impl ResponseIngester,
) -> ResponsePartBuilder<'ctx> {
    let endpoint = ctx.endpoint();

    let result = async {
        let body: Bytes = body.into();

        headers.typed_insert(headers::ContentType::json());
        headers.typed_insert(headers::ContentLength(body.len() as u64));

        headers.insert(
            http::header::ACCEPT,
            http::HeaderValue::from_static(
                "application/graphql-response+json; charset=utf-8, application/json; charset=utf-8",
            ),
        );

        let request = FetchRequest {
            websocket_init_payload: None,
            subgraph_name: endpoint.subgraph_name(),
            url: Cow::Owned(endpoint.url().clone()),
            headers,
            method: http::Method::POST,
            body,
            timeout: endpoint.config.timeout,
        };

        ctx.record_request_size(&request);

        let fetcher = ctx.runtime().fetcher();

        let fetch_result = retrying_fetch(ctx, || {
            let request = request.clone();
            let endpoint_url = endpoint.url().clone();
            let subgraph_name = endpoint.subgraph_name().to_string();

            async move {
                let http_span = SubgraphHttpRequestSpan::new(&endpoint_url, &http::Method::POST);
                let mut request = request.clone();

                grafbase_telemetry::otel::opentelemetry::global::get_text_map_propagator(|propagator| {
                    let context = http_span.context();
                    propagator.inject_context(
                        &context,
                        &mut grafbase_telemetry::http::HeaderInjector(&mut request.headers),
                    );
                });

                let (fetch_result, info) = fetcher.fetch(request).instrument(http_span.span()).await;

                let fetch_result = fetch_result.and_then(|response| {
                    tracing::debug!("Received response:\n{}", String::from_utf8_lossy(response.body()));
                    // For those status codes we want to retry the request, so marking the request as
                    // failed.
                    let status = response.status();
                    if status.is_server_error() || status == http::StatusCode::TOO_MANY_REQUESTS {
                        Err(FetchError::InvalidStatusCode(status))
                    } else {
                        Ok((response, info))
                    }
                });

                match fetch_result {
                    Ok((ref response, _)) => {
                        http_span.record_http_status_code(response.status());
                    }
                    Err(ref err) => {
                        tracing::error!("Request to subgraph {} failed with: {err}", subgraph_name);
                        http_span.set_as_http_error(err.as_invalid_status_code());
                    }
                };

                fetch_result
            }
        })
        .await;

        let fetch_result = fetch_result.map(|(response, info)| {
            if let Some(mut info) = info {
                info.headers(response.headers().clone());
                ctx.push_request_execution(RequestExecution::Response(info.build()));
            }
            response
        });

        match fetch_result {
            Ok(http_response) => {
                ctx.record_http_response(&http_response);
                // If the status code isn't a success as this point it means it's either a client error or
                // we've exhausted our retry budget for server errors.
                if http_response.status().is_success() {
                    Ok((http_response, ctx))
                } else {
                    tracing::debug!(
                        "Subgraph request failed with status code: {}\n{}",
                        http_response.status().as_u16(),
                        String::from_utf8_lossy(http_response.body())
                    );
                    Err(GraphqlError::new(
                        format!("Request failed with status code: {}", http_response.status().as_u16()),
                        ErrorCode::SubgraphRequestError,
                    ))
                }
            }
            Err(err) => {
                ctx.set_as_http_error(err.as_fetch_invalid_status_code());
                Err(err.into())
            }
        }
    };

    match result.await {
        Ok((response, ctx)) => {
            let (status, response_part) = ingester.ingest(Ok(response), response_part).await;

            if let Some(status) = status {
                ctx.set_graphql_response_status(status);
            } else {
                ctx.set_as_invalid_response();
            }

            response_part
        }
        Err(err) => {
            let (_, response_part) = ingester.ingest(Err(err), response_part).await;
            response_part
        }
    }
}

pub(crate) async fn retrying_fetch<R: Runtime, F, T>(
    ctx: &mut SubgraphContext<'_, R>,
    fetch: impl Fn() -> F + Send + Sync,
) -> ExecutionResult<T>
where
    F: Future<Output = FetchResult<T>> + Send,
    T: Send,
{
    let mut fetch_result = rate_limited_fetch(ctx, &fetch).instrument(Span::current()).await;

    if ctx.retry_budget().is_none() {
        return fetch_result;
    };

    let mut counter = 0;

    loop {
        match fetch_result {
            Ok(response) => {
                if let Some(b) = ctx.retry_budget() {
                    b.deposit()
                }
                return Ok(response);
            }
            Err(err) => {
                let withdraw = ctx.retry_budget().map(|b| b.withdraw()).unwrap_or_default();

                if withdraw {
                    let jitter = rand::random::<f64>() * 2.0;
                    let exp_backoff = (100 * 2u64.pow(counter)) as f64;
                    let backoff_ms = (exp_backoff * jitter).round() as u64;

                    ctx.engine().runtime.sleep(Duration::from_millis(backoff_ms)).await;
                    ctx.record_request_retry();

                    counter += 1;

                    fetch_result = rate_limited_fetch(ctx, &fetch).await;
                } else {
                    ctx.record_aborted_request_retry();

                    return Err(err);
                }
            }
        }
    }
}

async fn rate_limited_fetch<R: Runtime, F, T>(
    ctx: &mut SubgraphContext<'_, R>,
    fetch: impl Fn() -> F + Send,
) -> ExecutionResult<T>
where
    F: Future<Output = FetchResult<T>> + Send,
    T: Send,
{
    ctx.engine()
        .runtime
        .rate_limiter()
        .limit(&RateLimitKey::Subgraph(ctx.endpoint().subgraph_name().into()))
        .await
        .inspect_err(|_| {
            ctx.push_request_execution(RequestExecution::RateLimited);
        })?;

    ctx.increment_inflight_requests();
    let result = fetch().await;
    ctx.decrement_inflight_requests();

    if result.is_err() {
        ctx.push_request_execution(RequestExecution::RequestError);
    }

    result.map_err(|error| ExecutionError::Fetch {
        subgraph_name: ctx.endpoint().subgraph_name().to_string(),
        error,
    })
}
