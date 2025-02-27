#![allow(deprecated)]
use std::{borrow::Cow, time::Duration};

use bytes::Bytes;
use futures::Future;
use grafbase_telemetry::{
    graphql::GraphqlResponseStatus, otel::tracing_opentelemetry::OpenTelemetrySpanExt as _,
    span::subgraph::SubgraphHttpRequestSpan,
};
use headers::HeaderMapExt;
use runtime::{
    bytes::OwnedOrSharedBytes,
    fetch::{FetchError, FetchRequest, FetchResult, Fetcher},
    hooks::{ResponseInfo, SubgraphRequestExecutionKind},
    rate_limiting::RateLimitKey,
};
use tower::retry::budget::Budget;
use tracing::{Instrument, Span};

use crate::{
    Runtime,
    execution::{ExecutionError, ExecutionResult},
    resolver::graphql::SubgraphContext,
    response::{ErrorCode, GraphqlError, SubgraphResponse},
};

pub trait ResponseIngester: Send {
    fn ingest(
        self,
        response: http::Response<OwnedOrSharedBytes>,
    ) -> impl Future<Output = Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError>> + Send;
}

impl<F> ResponseIngester for F
where
    F: FnOnce(http::Response<OwnedOrSharedBytes>) -> ExecutionResult<(GraphqlResponseStatus, SubgraphResponse)> + Send,
{
    async fn ingest(
        self,
        response: http::Response<OwnedOrSharedBytes>,
    ) -> ExecutionResult<(GraphqlResponseStatus, SubgraphResponse)> {
        self(response)
    }
}

pub(crate) async fn execute_subgraph_request<R: Runtime>(
    ctx: &mut SubgraphContext<'_, R>,
    headers: http::HeaderMap,
    body: impl Into<Bytes> + Send,
    ingester: impl ResponseIngester,
) -> ExecutionResult<SubgraphResponse> {
    let endpoint = ctx.endpoint();

    let req = runtime::hooks::SubgraphRequest {
        method: http::Method::POST,
        url: endpoint.url().clone(),
        headers,
    };
    let runtime::hooks::SubgraphRequest {
        method,
        url,
        mut headers,
    } = ctx
        .hooks()
        .on_subgraph_request(endpoint.subgraph_name(), req)
        .await
        .inspect_err(|_| {
            ctx.set_as_hook_error();
            ctx.push_request_execution(SubgraphRequestExecutionKind::HookError);
        })?;

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
        url: Cow::Owned(url),
        headers,
        method,
        body,
        timeout: endpoint.config.timeout,
    };
    println!("== Request Headers ==");
    for (k, v) in request.headers.iter() {
        println!("{}: {}", k, v.to_str().unwrap());
    }
    println!("== Request Body ==");
    println!("{}", String::from_utf8_lossy(request.body.as_ref()));
    println!("\n");

    ctx.record_request_size(&request);

    let fetcher = ctx.engine.runtime.fetcher();
    let fetch_result = retrying_fetch(ctx, || async {
        let http_span = SubgraphHttpRequestSpan::new(endpoint.url(), &http::Method::POST);
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
                Ok(response)
            }
        });

        match fetch_result {
            Ok(ref response) => {
                http_span.record_http_status_code(response.status());
            }
            Err(ref err) => {
                http_span.set_as_http_error(err.as_invalid_status_code());
            }
        };

        (fetch_result, info)
    })
    .await;

    let response = match fetch_result {
        Ok(response) => {
            ctx.record_http_response(&response);
            response
        }
        Err(err) => {
            ctx.set_as_http_error(err.as_fetch_invalid_status_code());
            return Err(err);
        }
    };
    println!("== Response Headers ==");
    for (k, v) in response.headers().iter() {
        println!("{}: {}", k, v.to_str().unwrap());
    }
    println!("== Response Body ==\n{}", base64::encode(response.body().as_ref()));

    // If the status code isn't a success as this point it means it's either a client error or
    // we've exhausted our retry budget for server errors.
    if !response.status().is_success() {
        tracing::error!(
            "Subgraph request failed with status code: {}\n{}",
            response.status().as_u16(),
            String::from_utf8_lossy(response.body())
        );
        return Err(GraphqlError::new(
            format!("Request failed with status code: {}", response.status().as_u16()),
            ErrorCode::SubgraphRequestError,
        )
        .into());
    }

    match ingester.ingest(response).await {
        Ok((status, response)) => {
            ctx.set_graphql_response_status(status);
            Ok(response)
        }
        Err(err) => {
            ctx.set_as_invalid_response();
            Err(err)
        }
    }
}

pub(crate) async fn retrying_fetch<R: Runtime, F, T>(
    ctx: &mut SubgraphContext<'_, R>,
    fetch: impl Fn() -> F + Send + Sync,
) -> ExecutionResult<T>
where
    F: Future<Output = (FetchResult<T>, Option<ResponseInfo>)> + Send,
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
    F: Future<Output = (FetchResult<T>, Option<ResponseInfo>)> + Send,
    T: Send,
{
    ctx.engine()
        .runtime
        .rate_limiter()
        .limit(&RateLimitKey::Subgraph(ctx.endpoint().subgraph_name().into()))
        .await
        .inspect_err(|_| {
            ctx.push_request_execution(SubgraphRequestExecutionKind::RateLimited);
        })?;

    ctx.increment_inflight_requests();
    let (result, response_info) = fetch().await;
    ctx.decrement_inflight_requests();

    match response_info {
        Some(response_info) => ctx.push_request_execution(SubgraphRequestExecutionKind::Responsed(response_info)),
        None if result.is_err() => ctx.push_request_execution(SubgraphRequestExecutionKind::RequestError),
        None => (),
    }

    result.map_err(|error| ExecutionError::Fetch {
        subgraph_name: ctx.endpoint().subgraph_name().to_string(),
        error,
    })
}
