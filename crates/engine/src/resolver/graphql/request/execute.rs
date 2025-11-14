use std::{borrow::Cow, time::Duration};

use bytes::Bytes;
use event_queue::{RequestExecution, SubgraphResponseBuilder};
use futures::Future;
use grafbase_telemetry::{
    graphql::GraphqlResponseStatus, otel::tracing_opentelemetry::OpenTelemetrySpanExt as _,
    span::subgraph::SubgraphHttpRequestSpan,
};
use headers::HeaderMapExt;
use runtime::{
    extension::{EngineHooksExtension, ReqwestParts},
    fetch::{FetchError, FetchRequest, FetchResult, Fetcher},
    rate_limiting::RateLimitKey,
};
use tower::retry::budget::Budget;
use tracing::{Instrument, Span};

use crate::{
    EngineOperationContext, Runtime,
    execution::{ExecutionError, ExecutionResult},
    resolver::graphql::SubgraphContext,
    response::{GraphqlError, ResponsePartBuilder},
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
    headers: http::HeaderMap,
    is_mutation: bool,
    body: impl Into<Bytes> + Send,
    response_part: ResponsePartBuilder<'ctx>,
    ingester: impl ResponseIngester,
) -> ResponsePartBuilder<'ctx> {
    let subgraph = ctx.endpoint();

    let result = async {
        let ReqwestParts {
            url,
            method,
            mut headers,
        } = ctx
            .extensions()
            .on_graphql_subgraph_request(
                EngineOperationContext::from(&ctx.ctx),
                ctx.subgraph,
                ReqwestParts {
                    url: Cow::Borrowed(subgraph.url()),
                    method: http::Method::POST,
                    headers,
                },
            )
            .await?;

        let body: Bytes = body.into();

        headers.typed_insert(headers::ContentType::json());
        headers.typed_insert(headers::ContentLength(body.len() as u64));

        headers.insert(
            http::header::ACCEPT,
            http::HeaderValue::from_static(
                "application/graphql-response+json; charset=utf-8, application/json; charset=utf-8",
            ),
        );
        headers.insert(http::header::CONNECTION, http::HeaderValue::from_static("keep-alive"));

        let request = FetchRequest {
            subgraph_id: subgraph.id,
            url,
            is_mutation,
            headers,
            method,
            body,
            timeout: subgraph.config.timeout,
        };

        ctx.record_request_size(request.body.len());

        let fetcher = ctx.runtime().fetcher();

        let fetch_result = retrying_fetch(ctx, || {
            let mut request = request.clone();
            let subgraph_name = subgraph.name().to_string();

            async move {
                let http_span = SubgraphHttpRequestSpan::new(request.url.as_ref(), &http::Method::POST);

                grafbase_telemetry::otel::opentelemetry::global::get_text_map_propagator(|propagator| {
                    let context = http_span.context();
                    propagator.inject_context(
                        &context,
                        &mut grafbase_telemetry::http::HeaderInjector(&mut request.headers),
                    );
                });

                let (fetch_result, mut info) = fetcher.fetch(request).instrument(http_span.span()).await;

                let result = fetch_result.and_then(|mut response| {
                    tracing::debug!("Received response:\n{}", String::from_utf8_lossy(response.body()));
                    // For those status codes we want to retry the request, so marking the request as
                    // failed.
                    let status = response.status();

                    if let Some(ref mut info) = info {
                        info.status(status);

                        // Performance optimization: Instead of cloning the entire HeaderMap,
                        // we extract only the cache-related headers (Cache-Control and Age)
                        // that are needed by the caching logic. This avoids an expensive clone
                        // of all headers while still allowing telemetry/hooks to receive the
                        // complete header information.
                        let cache_control = response.headers().typed_get::<headers::CacheControl>();
                        let age = response.headers().typed_get::<headers::Age>();

                        // Move all headers to the hooks
                        info.headers(std::mem::take(response.headers_mut()));

                        // Put back cache-related headers for cache control logic
                        if let Some(cache_control) = cache_control {
                            response.headers_mut().typed_insert(cache_control);
                        }

                        if let Some(age) = age {
                            response.headers_mut().typed_insert(age);
                        }
                    }

                    if status == http::StatusCode::TOO_MANY_REQUESTS {
                        Err(FetchError::InvalidStatusCode(status))
                    } else {
                        Ok(response)
                    }
                });

                match result {
                    Ok(ref response) => {
                        http_span.record_http_status_code(response.status());
                    }
                    Err(ref err) => {
                        tracing::error!("Request to subgraph {} failed with: {err}", subgraph_name);
                        http_span.set_as_http_error(err.as_invalid_status_code());
                        // Only clear info for non-status-code errors (e.g., network errors)
                        // For status code errors, we want to preserve the response info
                        if !matches!(err, FetchError::InvalidStatusCode(_)) {
                            info = None;
                        }
                    }
                };

                (result, info)
            }
        })
        .await;

        match fetch_result {
            Ok(http_response) => {
                ctx.record_http_response(&http_response);
                // If the status code isn't a success as this point it means it's either a client error or
                // we've exhausted our retry budget for server errors.
                if !http_response.status().is_success() {
                    tracing::debug!(
                        "Subgraph request failed with status code: {}\n{}",
                        http_response.status().as_u16(),
                        String::from_utf8_lossy(http_response.body())
                    );
                }
                Ok((http_response, ctx))
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
    F: Future<Output = (FetchResult<T>, Option<SubgraphResponseBuilder>)> + Send,
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
    F: Future<Output = (FetchResult<T>, Option<SubgraphResponseBuilder>)> + Send,
    T: Send,
{
    ctx.engine()
        .runtime
        .rate_limiter()
        .limit(&RateLimitKey::Subgraph(ctx.endpoint().name().into()))
        .await
        .inspect_err(|_| {
            ctx.push_request_execution(RequestExecution::RateLimited);
        })?;

    ctx.increment_inflight_requests();
    let (result, info) = fetch().await;
    ctx.decrement_inflight_requests();

    match info {
        Some(info) => ctx.push_request_execution(RequestExecution::Response(info.build())),
        None if result.is_err() => ctx.push_request_execution(RequestExecution::RequestError),
        None => (),
    }

    result.map_err(|error| ExecutionError::Fetch {
        subgraph_name: ctx.endpoint().name().to_string(),
        error,
    })
}
