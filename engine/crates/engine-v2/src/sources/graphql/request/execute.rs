use std::borrow::Cow;

use bytes::Bytes;
use futures::Future;
use grafbase_telemetry::{graphql::GraphqlResponseStatus, otel::tracing_opentelemetry::OpenTelemetrySpanExt as _};
use headers::HeaderMapExt;
use runtime::{
    bytes::OwnedOrSharedBytes,
    fetch::{FetchError, FetchRequest, FetchResult, Fetcher},
    hooks::{ResponseInfo, SubgraphRequestExecutionKind},
    rate_limiting::RateLimitKey,
};
use web_time::Duration;

use crate::{
    execution::{ExecutionError, ExecutionResult},
    response::{GraphqlError, SubgraphResponse},
    sources::graphql::SubgraphContext,
    ErrorCode, Runtime,
};

pub trait ResponseIngester: Send {
    /// Processes the HTTP response from a subgraph request.
    ///
    /// This function ingests the given response and returns a future that resolves to a
    /// result containing the GraphQL response status and the subgraph response. In the event
    /// of an error during ingestion, an `ExecutionError` will be returned.
    ///
    /// # Parameters
    ///
    /// * `response`: The HTTP response that needs to be ingested.
    ///
    /// # Returns
    ///
    /// A future that, when resolved, yields a `Result` containing a tuple of `GraphqlResponseStatus`
    /// and `SubgraphResponse` on success or an `ExecutionError` on failure.
    fn ingest(
        self,
        response: http::Response<OwnedOrSharedBytes>,
    ) -> impl Future<Output = Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError>> + Send;
}

/// Executes a subgraph request asynchronously.
///
/// This function constructs a fetch request to the specified subgraph, sends it, and processes
/// the HTTP response. It uses a provided `ResponseIngester` to handle the response. If an error
/// occurs during the process, an `ExecutionError` is returned.
///
/// # Type Parameters
///
/// * `'ctx`: A lifetime parameter representing the context lifetime.
/// * `'a`: A lifetime parameter that can be used for associated references.
/// * `R`: The engine runtime.
///
/// # Parameters
///
/// * `ctx`: A mutable reference to the context for the subgraph request execution.
/// * `headers`: The HTTP headers to include in the request.
/// * `body`: The request body as `Bytes`.
/// * `ingester`: A value implementing the `ResponseIngester` trait used to process the response.
///
/// # Returns
///
/// An `ExecutionResult` which resolves to a `SubgraphResponse` on success or an `ExecutionError` on failure.
pub(crate) async fn execute_subgraph_request<'ctx, 'a, R: Runtime>(
    ctx: &mut SubgraphContext<'ctx, R>,
    headers: http::HeaderMap,
    body: Bytes,
    ingester: impl ResponseIngester,
) -> ExecutionResult<SubgraphResponse> {
    let endpoint = ctx.endpoint();

    let request = {
        let mut headers = ctx
            .hooks()
            .on_subgraph_request(endpoint.subgraph_name(), http::Method::POST, endpoint.url(), headers)
            .await
            .inspect_err(|_| {
                ctx.set_as_hook_error();
                ctx.push_request_execution(SubgraphRequestExecutionKind::HookError);
            })?;

        headers.typed_insert(headers::ContentType::json());
        headers.typed_insert(headers::ContentLength(body.len() as u64));
        headers.insert(http::header::ACCEPT, http::HeaderValue::from_static("application/json"));

        grafbase_telemetry::otel::opentelemetry::global::get_text_map_propagator(|propagator| {
            let context = tracing::Span::current().context();

            propagator.inject_context(&context, &mut grafbase_telemetry::http::HeaderInjector(&mut headers));
        });

        FetchRequest {
            url: Cow::Borrowed(endpoint.url()),
            headers,
            method: http::Method::POST,
            body,
            timeout: endpoint.config.timeout,
        }
    };

    ctx.record_request(&request);

    let fetcher = ctx.engine.runtime.fetcher();
    let fetch_result = retrying_fetch(ctx, || async {
        let (fetch_result, info) = fetcher.fetch(request.clone()).await;
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

    // If the status code isn't a success as this point it means it's either a client error or
    // we've exhausted our retry budget for server errors.
    if !response.status().is_success() {
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
            tracing::debug!("invalid subgraph response: {err}");
            Err(err)
        }
    }
}

/// Attempts to fetch a request with built-in retry logic.
///
/// This function tries to execute a fetch request and will retry the request in certain conditions
/// such as network errors or temporary server errors. The number of retries is governed by the retry
/// budget from the context. The function also implements exponential backoff for retries to prevent
/// overwhelming the server with requests.
///
/// # Type Parameters
///
/// * `'ctx`: A lifetime parameter representing the context lifetime.
/// * `R`: The engine runtime.
/// * `F`: A future type that represents the fetch operation.
/// * `T`: The type of the result produced by the fetch.
///
/// # Parameters
///
/// * `ctx`: A mutable reference to the context for the subgraph request execution.
/// * `fetch`: A function that returns a future representing the fetch operation.
///
/// # Returns
///
/// An `ExecutionResult` which resolves to the outcome of the fetch operation on success,
/// or an `ExecutionError` if the fetch operations fail after exhausting the retry budget.
pub(crate) async fn retrying_fetch<'ctx, R: Runtime, F, T>(
    ctx: &mut SubgraphContext<'ctx, R>,
    fetch: impl Fn() -> F + Send + Sync,
) -> ExecutionResult<T>
where
    F: Future<Output = (FetchResult<T>, Option<ResponseInfo>)> + Send,
    T: Send,
{
    let mut fetch_result = rate_limited_fetch(ctx, &fetch).await;

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
                let withdraw = ctx.retry_budget().and_then(|b| b.withdraw().ok());

                if withdraw.is_some() {
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

/// Attempts to fetch a request while adhering to rate limits.
///
/// This function performs a fetch operation and ensures that it respects the rate limits
/// defined for the subgraph.
///
/// # Type Parameters
///
/// * `'ctx`: A lifetime parameter representing the context lifetime.
/// * `R`: The engine runtime.
/// * `F`: A future type that represents the fetch operation.
/// * `T`: The type of the result produced by the fetch.
///
/// # Parameters
///
/// * `ctx`: A mutable reference to the context for the subgraph request execution.
/// * `fetch`: A function that returns a future representing the fetch operation.
///
/// # Returns
///
/// An `ExecutionResult` which resolves to the outcome of the fetch operation on success,
/// or an `ExecutionError` if the fetch operation fails.
async fn rate_limited_fetch<'ctx, R: Runtime, F, T>(
    ctx: &mut SubgraphContext<'ctx, R>,
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
