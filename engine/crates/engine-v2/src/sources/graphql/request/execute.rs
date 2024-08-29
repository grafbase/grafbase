use std::borrow::Cow;

use bytes::Bytes;
use futures::Future;
use grafbase_telemetry::{
    gql_response_status::{GraphqlResponseStatus, SubgraphResponseStatus},
    span::{GqlRecorderSpanExt, GRAFBASE_TARGET},
};
use headers::HeaderMapExt;
use runtime::{
    bytes::OwnedOrSharedBytes,
    fetch::{FetchRequest, FetchResult, Fetcher},
    hooks::{ResponseInfo, ResponseKind},
    rate_limiting::RateLimitKey,
};
use schema::sources::graphql::GraphqlEndpointId;
use tower::retry::budget::Budget;
use tracing::Span;
use web_time::{Duration, SystemTime};

use crate::{
    execution::{ExecutionContext, ExecutionError, ExecutionResult},
    response::SubgraphResponse,
    sources::{graphql::record, SubgraphRequestContext},
    Runtime,
};

pub trait ResponseIngester: Send {
    fn ingest(
        self,
        bytes: http::Response<OwnedOrSharedBytes>,
    ) -> impl Future<Output = Result<(GraphqlResponseStatus, SubgraphResponse), ExecutionError>> + Send;
}

pub(crate) struct SubgraphRequest<'ctx, 'a, R: Runtime> {
    pub ctx: ExecutionContext<'ctx, R>,
    pub span: Span,
    pub endpoint_id: GraphqlEndpointId,
    pub retry_budget: Option<&'a Budget>,
    pub headers: http::HeaderMap,
    pub body: Bytes,
}

pub(crate) async fn execute_subgraph_request<'ctx, 'a, R: Runtime>(
    ctx: &mut SubgraphRequestContext<'ctx, R>,
    span: Span,
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
            .map_err(|error| {
                ctx.request_info().push_response(ResponseKind::HookError);
                error
            })?;

        headers.typed_insert(headers::ContentType::json());
        headers.typed_insert(headers::ContentLength(body.len() as u64));
        headers.insert(http::header::ACCEPT, http::HeaderValue::from_static("application/json"));

        FetchRequest {
            url: Cow::Borrowed(endpoint.url()),
            headers,
            method: http::Method::POST,
            body,
            timeout: endpoint.timeout(),
        }
    };

    let start = SystemTime::now();
    let execution_context = ctx.execution_context();

    let response = retrying_fetch(ctx, move || {
        record::subgraph_request_size(execution_context, endpoint, request.body.len());
        execution_context.engine.runtime.fetcher().fetch(request.clone())
    })
    .await;

    let duration = SystemTime::now()
        .duration_since(start)
        .unwrap_or(Duration::from_secs(0));

    let response = match response {
        Ok(response) => response,
        Err(e) => {
            let status = SubgraphResponseStatus::HttpError;

            ctx.request_info().set_graphql_status(status);
            record::subgraph_duration(execution_context, endpoint, status, None, duration);

            return Err(e);
        }
    };

    tracing::debug!("{}", String::from_utf8_lossy(response.body()));
    record::subgraph_response_size(execution_context, endpoint, response.body().len());

    let status_code = response.status();

    let (status, response) = ingester.ingest(response).await.inspect_err(|err| {
        let subgraph_status = SubgraphResponseStatus::InvalidResponseError;

        ctx.request_info().set_graphql_status(subgraph_status);
        span.record_subgraph_status(subgraph_status);

        record::subgraph_duration(
            execution_context,
            endpoint,
            subgraph_status,
            Some(status_code),
            duration,
        );

        tracing::error!(target: GRAFBASE_TARGET, "{err}");
    })?;

    let subgraph_status = SubgraphResponseStatus::GraphqlResponse(status);
    ctx.request_info().set_graphql_status(subgraph_status);

    span.record_subgraph_status(subgraph_status);

    record::subgraph_duration(
        execution_context,
        endpoint,
        subgraph_status,
        Some(status_code),
        duration,
    );

    match response.subgraph_errors().next().map(|e| &e.message) {
        Some(error) => {
            tracing::error!(target: GRAFBASE_TARGET, "{error}");
        }
        None => {
            tracing::debug!(target: GRAFBASE_TARGET, "subgraph request")
        }
    }

    Ok(response)
}

async fn retrying_fetch<'ctx, R: Runtime, F, T>(
    ctx: &mut SubgraphRequestContext<'ctx, R>,
    fetch: impl Fn() -> F + Send + Sync,
) -> ExecutionResult<http::Response<T>>
where
    F: Future<Output = FetchResult<http::Response<T>>> + Send,
    T: Send,
{
    let mut result = rate_limited_fetch(ctx, &fetch).await;

    if ctx.retry_budget().is_none() {
        return result;
    };

    let mut counter = 0;

    loop {
        match result {
            Ok(bytes) => {
                ctx.retry_budget().map(|b| b.deposit());
                return Ok(bytes);
            }
            Err(err) => {
                let withdraw = ctx.retry_budget().and_then(|b| b.withdraw().ok());

                if withdraw.is_some() {
                    let jitter = rand::random::<f64>() * 2.0;
                    let exp_backoff = (100 * 2u64.pow(counter)) as f64;
                    let backoff_ms = (exp_backoff * jitter).round() as u64;

                    ctx.engine().runtime.sleep(Duration::from_millis(backoff_ms)).await;
                    record::subgraph_retry(ctx.execution_context(), ctx.endpoint(), false);

                    counter += 1;

                    result = rate_limited_fetch(ctx, &fetch).await;
                } else {
                    record::subgraph_retry(ctx.execution_context(), ctx.endpoint(), true);

                    return Err(err);
                }
            }
        }
    }
}

async fn rate_limited_fetch<'ctx, R: Runtime, F, T>(
    ctx: &mut SubgraphRequestContext<'ctx, R>,
    fetch: impl Fn() -> F + Send,
) -> ExecutionResult<http::Response<T>>
where
    F: Future<Output = FetchResult<http::Response<T>>> + Send,
    T: Send,
{
    ctx.engine()
        .runtime
        .rate_limiter()
        .limit(&RateLimitKey::Subgraph(ctx.endpoint().subgraph_name().into()))
        .await
        .map_err(|e| {
            ctx.request_info().push_response(ResponseKind::RateLimited);
            e
        })?;

    record::increment_inflight_requests(ctx.execution_context(), ctx.endpoint());
    let mut result = fetch().await;
    record::decrement_inflight_requests(ctx.execution_context(), ctx.endpoint());

    match result {
        Ok(ref mut response) => {
            if let Some(info) = response.extensions_mut().remove::<ResponseInfo>() {
                ctx.request_info().push_response(ResponseKind::Responsed(info));
            }
        }
        Err(_) => {
            ctx.request_info().push_response(ResponseKind::RequestError);
        }
    }

    result.map_err(|error| ExecutionError::Fetch {
        subgraph_name: ctx.endpoint().subgraph_name().to_string(),
        error,
    })
}
