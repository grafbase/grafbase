use bytes::Bytes;
use grafbase_telemetry::{
    graphql::GraphqlResponseStatus,
    span::subgraph::{SubgraphGraphqlRequestSpan, SubgraphRequestSpanBuilder},
};
use runtime::{
    bytes::OwnedOrSharedBytes,
    fetch::FetchRequest,
    hooks::{CacheStatus, ExecutedSubgraphRequest, ExecutedSubgraphRequestBuilder, SubgraphRequestExecutionKind},
};
use schema::GraphqlEndpoint;
use std::ops::Deref;
use tower::retry::budget::Budget;
use tracing::Span;
use web_time::Instant;

use grafbase_telemetry::{
    graphql::SubgraphResponseStatus,
    metrics::{
        SubgraphCacheHitAttributes, SubgraphCacheMissAttributes, SubgraphInFlightRequestAttributes,
        SubgraphRequestBodySizeAttributes, SubgraphRequestDurationAttributes, SubgraphRequestRetryAttributes,
        SubgraphResponseBodySizeAttributes,
    },
};

use crate::{
    execution::{ExecutionContext, ExecutionError, ExecutionResult, RequestHooks},
    response::SubgraphResponse,
    sources::ResolverResult,
    Engine, Runtime,
};

#[derive(Clone)]
/// Context for executing a subgraph request.
///
/// This struct holds relevant information about the execution context,
/// including the execution context itself, the GraphQL endpoint being
/// queried, and metrics for tracking request status and performance.
pub(crate) struct SubgraphContext<'ctx, R: Runtime> {
    /// The execution context in which the subgraph operates.
    pub(super) ctx: ExecutionContext<'ctx, R>,

    /// The GraphQL endpoint being utilized for the request.
    pub(super) endpoint: GraphqlEndpoint<'ctx>,

    /// An optional retry budget that indicates the amount of retries allowed.
    pub(super) retry_budget: Option<&'ctx Budget>,

    /// Span for tracking the GraphQL request within telemetry.
    span: SubgraphGraphqlRequestSpan,

    /// The start time of the request execution.
    start: Instant,

    /// Builder for constructing data for the `on-operation-response` hook.
    executed_request_builder: ExecutedSubgraphRequestBuilder<'ctx>,

    /// The status of the subgraph response, if available.
    status: Option<SubgraphResponseStatus>,

    /// The HTTP status code received from the subgraph response, if applicable.
    http_status_code: Option<http::StatusCode>,

    /// The count of times the request has been sent.
    send_count: usize,
}

impl<'ctx, R: Runtime> Deref for SubgraphContext<'ctx, R> {
    type Target = ExecutionContext<'ctx, R>;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<'ctx, R: Runtime> SubgraphContext<'ctx, R> {
    /// Creates a new instance of `SubgraphContext`.
    ///
    /// This function initializes a `SubgraphContext` with the specified execution context,
    /// GraphQL endpoint, and telemetry span builder. It retrieves the appropriate retry budget
    /// based on the type of operation being performed (mutation or non-mutation).
    ///
    /// # Parameters
    ///
    /// - `ctx`: The execution context for the subgraph operation.
    /// - `endpoint`: The GraphQL endpoint to be queried.
    /// - `span`: A builder for creating the telemetry span.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `SubgraphContext` configured with the provided values.
    pub fn new(
        ctx: ExecutionContext<'ctx, R>,
        endpoint: GraphqlEndpoint<'ctx>,
        span: SubgraphRequestSpanBuilder<'_>,
    ) -> Self {
        let executed_request_builder =
            ExecutedSubgraphRequest::builder(endpoint.subgraph_name(), "POST", endpoint.url().as_str());

        let retry_budget = match span.operation_type {
            "mutation" => ctx.engine.get_retry_budget_for_mutation(endpoint.id()),
            _ => ctx.engine.get_retry_budget_for_non_mutation(endpoint.id()),
        };
        let span = span.build();

        Self {
            ctx,
            endpoint,
            executed_request_builder,
            span,
            start: Instant::now(),
            retry_budget,
            status: None,
            http_status_code: None,
            send_count: 0,
        }
    }

    /// Retrieves the execution context associated with this subgraph context.
    ///
    /// This function provides access to the `ExecutionContext` that encapsulates
    /// the environment in which the subgraph operates, including any relevant
    /// configuration and state needed for executing GraphQL operations.
    ///
    /// # Returns
    ///
    /// A reference to the `ExecutionContext` for the subgraph operation.
    pub fn execution_context(&self) -> ExecutionContext<'ctx, R> {
        self.ctx
    }

    /// Retrieves the telemetry span associated with this subgraph request.
    ///
    /// This function provides access to the telemetry span that tracks
    /// the GraphQL request.
    ///
    /// # Returns
    ///
    /// A reference to the `Span` object used for telemetry.
    pub fn span(&self) -> Span {
        self.span.span.clone()
    }

    /// Retrieves a reference to the engine used during the execution
    /// of this subgraph context.
    ///
    /// # Returns
    ///
    /// A reference to the `Engine` associated with the runtime.
    pub fn engine(&self) -> &Engine<R> {
        self.execution_context().engine
    }

    /// Retrieves the GraphQL endpoint being used for the request.
    ///
    /// # Returns
    ///
    /// A `GraphqlEndpoint` instance representing the endpoint used
    /// for the subgraph operation.
    pub fn endpoint(&self) -> GraphqlEndpoint<'ctx> {
        self.endpoint
    }

    /// Retrieves the request hooks associated with this subgraph context.
    ///
    /// This method returns the hooks that can be used to customize the execution
    /// behavior and integration of subgraph operations.
    ///
    /// # Returns
    ///
    /// A `RequestHooks` instance representing the hooks available for the current
    /// execution context.
    pub fn hooks(&self) -> RequestHooks<'ctx, R::Hooks> {
        self.execution_context().hooks()
    }

    /// Retrieves the optional retry budget associated with this subgraph context.
    ///
    /// This function provides access to the retry budget, which indicates
    /// the number of retries that are allowed for the current subgraph request.
    ///
    /// # Returns
    ///
    /// An `Option` containing a reference to the `Budget` if available, or `None`
    /// if no retry budget is set.
    pub fn retry_budget(&self) -> Option<&Budget> {
        self.retry_budget
    }

    /// Finalizes the subgraph context by processing the result of the subgraph execution.
    ///
    /// This method takes the result of the subgraph execution and performs necessary actions
    /// based on the outcome, including recording metrics and invoking the `on-subgraph-response`
    /// hook.
    ///
    /// # Parameters
    ///
    /// - `subgraph_result`: The result of the subgraph execution, encapsulated in an
    ///   `ExecutionResult<SubgraphResponse>`. This can either be a successful response
    ///   or an error.
    ///
    /// # Returns
    ///
    /// A `ResolverResult` containing the final execution result along with any output
    /// from the on-response hooks.
    pub async fn finalize(self, subgraph_result: ExecutionResult<SubgraphResponse>) -> ResolverResult {
        let duration = self.start.elapsed();

        if let Some(status) = self.status {
            self.span.record_graphql_response_status(status);
            self.metrics().record_subgraph_request_duration(
                SubgraphRequestDurationAttributes {
                    name: self.endpoint.subgraph_name().to_string(),
                    status,
                    http_status_code: self.http_status_code,
                },
                duration,
            );
        }

        if let Some(resend_count) = self.send_count.checked_sub(1) {
            self.span.record_resend_count(resend_count)
        }

        let hook_result = self
            .ctx
            .hooks()
            .on_subgraph_response(self.executed_request_builder.build(duration))
            .await
            .map_err(|e| {
                tracing::error!("error in on-subgraph-response hook: {e}");
                ExecutionError::Internal("internal error".into())
            });

        match hook_result {
            Ok(hook_result) => ResolverResult {
                execution: subgraph_result,
                on_subgraph_response_hook_output: Some(hook_result),
            },
            Err(e) => ResolverResult {
                execution: Err(e),
                on_subgraph_response_hook_output: None,
            },
        }
    }

    /// Increments the count of inflight requests for this subgraph context.
    ///
    /// This function updates the internal state to reflect that a new request
    /// is currently in flight. It also records metrics related to the inflight
    /// status of the subgraph request.
    pub(super) fn increment_inflight_requests(&mut self) {
        self.send_count += 1;
        self.metrics()
            .increment_subgraph_inflight_requests(SubgraphInFlightRequestAttributes {
                name: self.endpoint.subgraph_name().to_string(),
            });
    }

    /// Decrements the count of inflight requests for this subgraph context.
    ///
    /// This function updates the internal state to reflect that a request
    /// has completed and is no longer in flight. It also records metrics
    /// related to the decrease in inflight requests.
    pub(super) fn decrement_inflight_requests(&mut self) {
        self.metrics()
            .decrement_subgraph_inflight_requests(SubgraphInFlightRequestAttributes {
                name: self.endpoint.subgraph_name().to_string(),
            });
    }

    /// Records a cache hit for the subgraph request.
    ///
    /// This method updates the metrics and sets the cache status to `Hit`,
    /// indicating that the response was fully retrieved from the cache.
    pub(super) fn record_cache_hit(&mut self) {
        self.executed_request_builder.set_cache_status(CacheStatus::Hit);
        self.metrics().record_subgraph_cache_hit(SubgraphCacheHitAttributes {
            name: self.endpoint.subgraph_name().to_string(),
        });
    }

    /// Records a partial cache hit for the subgraph request.
    ///
    /// This method updates the metrics and sets the cache status to `PartialHit`,
    /// indicating that only part of the response was retrieved from the cache.
    pub(super) fn record_cache_partial_hit(&mut self) {
        self.executed_request_builder.set_cache_status(CacheStatus::PartialHit);
        self.metrics()
            .record_subgraph_cache_partial_hit(self.endpoint.subgraph_name().to_string());
    }

    /// Records a cache miss for the subgraph request.
    ///
    /// This method updates the metrics and sets the cache status to `Miss`,
    /// indicating that the requested data was not found in the cache.
    pub(super) fn record_cache_miss(&mut self) {
        self.executed_request_builder.set_cache_status(CacheStatus::Miss);
        self.metrics().record_subgraph_cache_miss(SubgraphCacheMissAttributes {
            name: self.endpoint.subgraph_name().to_string(),
        });
    }

    /// Records a subgraph request for metrics.
    ///
    /// This method captures the details of a `FetchRequest`, including its URL,
    /// method, and body size. It updates telemetry spans and metrics to reflect
    /// the request's size and attributes.
    ///
    /// # Parameters
    ///
    /// - `request`: A reference to the `FetchRequest` that is being recorded.
    pub(super) fn record_request(&mut self, request: &FetchRequest<'_, Bytes>) {
        self.span.record_http_request(&request.url, &request.method);
        self.metrics().record_subgraph_request_size(
            SubgraphRequestBodySizeAttributes {
                name: self.endpoint.subgraph_name().to_string(),
            },
            request.body.len(),
        );
    }

    /// Records a retry for an aborted request.
    ///
    /// This method updates the metrics to reflect a retry attempt
    /// that occurred due to the previous request being aborted.
    pub(super) fn record_aborted_request_retry(&self) {
        self.metrics().record_subgraph_retry(SubgraphRequestRetryAttributes {
            name: self.endpoint.subgraph_name().to_string(),
            aborted: true,
        });
    }

    /// Records a retry attempt for the subgraph request.
    ///
    /// This method updates the metrics to reflect a retry attempt
    /// that occurred following a previous request.
    pub(super) fn record_request_retry(&self) {
        self.metrics().record_subgraph_retry(SubgraphRequestRetryAttributes {
            name: self.endpoint.subgraph_name().to_string(),
            aborted: false,
        });
    }

    /// Pushes a new subgraph execution result, which will be available in the
    /// `on-subgraph-response` hook.
    pub(super) fn push_request_execution(&mut self, kind: SubgraphRequestExecutionKind) {
        self.executed_request_builder.push_execution(kind)
    }

    /// Records the HTTP response for the subgraph request.
    ///
    /// This method updates the telemetry span with the status code of the
    /// response and records the size of the response body. It also stores
    /// the HTTP status code for hooks.
    pub(super) fn record_http_response(&mut self, response: &http::Response<OwnedOrSharedBytes>) {
        self.span.record_http_status_code(response.status());
        self.http_status_code = Some(response.status());

        self.metrics().record_subgraph_response_size(
            SubgraphResponseBodySizeAttributes {
                name: self.endpoint.subgraph_name().to_string(),
            },
            response.body().len(),
        );
    }

    /// Marks the subgraph context as having encountered a hook error.
    ///
    /// This method updates the response status to indicate that an error occurred
    /// within one of the request hooks, affecting the outcome of the subgraph request.
    pub(super) fn set_as_hook_error(&mut self) {
        self.status = Some(SubgraphResponseStatus::HookError);
    }

    /// Marks the subgraph context as having encountered an HTTP error.
    ///
    /// This method updates the response status to indicate an HTTP error
    /// occurred during the subgraph request. It records the provided HTTP
    /// status code and updates the telemetry span accordingly.
    ///
    /// Sets the execution as failed for the hook.
    ///
    /// # Parameters
    ///
    /// - `status_code`: An optional `http::StatusCode` representing the
    ///   specific HTTP error encountered. If provided, this status code
    ///   is recorded for telemetry.
    pub(super) fn set_as_http_error(&mut self, status_code: Option<http::StatusCode>) {
        if let Some(status_code) = status_code {
            self.span.record_http_status_code(status_code);
            self.http_status_code = Some(status_code);
        }

        self.status = Some(SubgraphResponseStatus::HttpError);
    }

    /// Marks the subgraph context as having received an invalid GraphQL response.
    ///
    /// This method updates the response status to indicate that the response
    /// from the subgraph was not in the expected format, which may affect
    /// further processing of the request.
    pub(super) fn set_as_invalid_response(&mut self) {
        self.status = Some(SubgraphResponseStatus::InvalidGraphqlResponseError);
    }

    /// Sets the GraphQL response status for the subgraph context.
    ///
    /// This method updates the internal status to reflect the provided
    /// GraphQL response status, indicating whether the response was well-formed.
    /// It also stores the status for the hook.
    ///
    /// # Parameters
    ///
    /// - `status`: The `GraphqlResponseStatus` representing the status of the response.
    pub(super) fn set_graphql_response_status(&mut self, status: GraphqlResponseStatus) {
        self.status = Some(SubgraphResponseStatus::WellFormedGraphqlResponse(status));
        self.executed_request_builder.set_graphql_response_status(status);
    }
}
