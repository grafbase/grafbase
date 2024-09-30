use bytes::Bytes;
use grafbase_telemetry::{
    graphql::GraphqlResponseStatus,
    span::subgraph::{SubgraphGraphqlRequestSpan, SubgraphHttpRequestSpan, SubgraphRequestSpanBuilder},
};
use runtime::{
    bytes::OwnedOrSharedBytes,
    fetch::FetchRequest,
    hooks::{CacheStatus, ExecutedSubgraphRequest, ExecutedSubgraphRequestBuilder, SubgraphRequestExecutionKind},
};
use schema::GraphqlEndpoint;
use std::ops::Deref;
use tower::retry::budget::TpsBudget;
use tracing::{Instrument, Span};
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
pub(crate) struct SubgraphContext<'ctx, R: Runtime> {
    pub(super) ctx: ExecutionContext<'ctx, R>,
    pub(super) endpoint: GraphqlEndpoint<'ctx>,
    pub(super) retry_budget: Option<&'ctx TpsBudget>,
    span: SubgraphGraphqlRequestSpan,
    start: Instant,
    executed_request_builder: ExecutedSubgraphRequestBuilder<'ctx>,
    status: Option<SubgraphResponseStatus>,
    http_status_code: Option<http::StatusCode>,
    send_count: usize,
}

impl<'ctx, R: Runtime> Deref for SubgraphContext<'ctx, R> {
    type Target = ExecutionContext<'ctx, R>;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<'ctx, R: Runtime> SubgraphContext<'ctx, R> {
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

    pub fn execution_context(&self) -> ExecutionContext<'ctx, R> {
        self.ctx
    }

    pub fn span(&self) -> Span {
        self.span.span.clone()
    }

    pub fn engine(&self) -> &Engine<R> {
        self.execution_context().engine
    }

    pub fn endpoint(&self) -> GraphqlEndpoint<'ctx> {
        self.endpoint
    }

    pub fn hooks(&self) -> RequestHooks<'ctx, R::Hooks> {
        self.execution_context().hooks()
    }

    pub fn retry_budget(&self) -> Option<&TpsBudget> {
        self.retry_budget
    }

    pub fn send_count(&self) -> Option<usize> {
        self.send_count.checked_sub(1)
    }

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

        let hook_result = self
            .ctx
            .hooks()
            .on_subgraph_response(self.executed_request_builder.build(duration))
            .instrument(self.span.span.clone())
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

    pub(super) fn increment_inflight_requests(&mut self) {
        self.send_count += 1;
        self.metrics()
            .increment_subgraph_inflight_requests(SubgraphInFlightRequestAttributes {
                name: self.endpoint.subgraph_name().to_string(),
            });
    }

    pub(super) fn decrement_inflight_requests(&mut self) {
        self.metrics()
            .decrement_subgraph_inflight_requests(SubgraphInFlightRequestAttributes {
                name: self.endpoint.subgraph_name().to_string(),
            });
    }

    pub(super) fn record_cache_hit(&mut self) {
        self.executed_request_builder.set_cache_status(CacheStatus::Hit);
        self.metrics().record_subgraph_cache_hit(SubgraphCacheHitAttributes {
            name: self.endpoint.subgraph_name().to_string(),
        });
    }

    pub(super) fn record_cache_partial_hit(&mut self) {
        self.executed_request_builder.set_cache_status(CacheStatus::PartialHit);
        self.metrics()
            .record_subgraph_cache_partial_hit(self.endpoint.subgraph_name().to_string());
    }

    pub(super) fn record_cache_miss(&mut self) {
        self.executed_request_builder.set_cache_status(CacheStatus::Miss);
        self.metrics().record_subgraph_cache_miss(SubgraphCacheMissAttributes {
            name: self.endpoint.subgraph_name().to_string(),
        });
    }

    pub(super) fn create_subgraph_request_span<T>(&self, request: &FetchRequest<'_, T>) -> SubgraphHttpRequestSpan {
        SubgraphHttpRequestSpan::new(&request.url, &request.method)
    }

    pub(super) fn record_request_size(&mut self, request: &FetchRequest<'_, Bytes>) {
        self.metrics().record_subgraph_request_size(
            SubgraphRequestBodySizeAttributes {
                name: self.endpoint.subgraph_name().to_string(),
            },
            request.body.len(),
        );
    }

    pub(super) fn record_aborted_request_retry(&self) {
        self.metrics().record_subgraph_retry(SubgraphRequestRetryAttributes {
            name: self.endpoint.subgraph_name().to_string(),
            aborted: true,
        });
    }

    pub(super) fn record_request_retry(&self) {
        self.metrics().record_subgraph_retry(SubgraphRequestRetryAttributes {
            name: self.endpoint.subgraph_name().to_string(),
            aborted: false,
        });
    }

    pub(super) fn push_request_execution(&mut self, kind: SubgraphRequestExecutionKind) {
        self.executed_request_builder.push_execution(kind)
    }

    pub(super) fn record_http_response(&mut self, response: &http::Response<OwnedOrSharedBytes>) {
        self.http_status_code = Some(response.status());
        self.metrics().record_subgraph_response_size(
            SubgraphResponseBodySizeAttributes {
                name: self.endpoint.subgraph_name().to_string(),
            },
            response.body().len(),
        );
    }

    pub(super) fn set_as_hook_error(&mut self) {
        self.status = Some(SubgraphResponseStatus::HookError);
    }

    pub(super) fn set_as_http_error(&mut self, status_code: Option<http::StatusCode>) {
        if let Some(status_code) = status_code {
            self.http_status_code = Some(status_code);
        }
        self.status = Some(SubgraphResponseStatus::HttpError);
    }

    pub(super) fn set_as_invalid_response(&mut self) {
        self.status = Some(SubgraphResponseStatus::InvalidGraphqlResponseError);
    }

    pub(super) fn set_graphql_response_status(&mut self, status: GraphqlResponseStatus) {
        self.status = Some(SubgraphResponseStatus::WellFormedGraphqlResponse(status));
        self.executed_request_builder.set_graphql_response_status(status);
    }
}
