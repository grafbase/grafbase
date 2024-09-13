use grafbase_telemetry::{
    graphql::SubgraphResponseStatus,
    metrics::{
        SubgraphCacheHitAttributes, SubgraphCacheMissAttributes, SubgraphInFlightRequestAttributes,
        SubgraphRequestBodySizeAttributes, SubgraphRequestDurationAttributes, SubgraphRequestRetryAttributes,
        SubgraphResponseBodySizeAttributes,
    },
};
use schema::GraphqlEndpoint;
use web_time::Duration;

use crate::{execution::ExecutionContext, Runtime};

pub(super) fn subgraph_retry<R: Runtime>(ctx: ExecutionContext<'_, R>, endpoint: GraphqlEndpoint<'_>, aborted: bool) {
    ctx.metrics().record_subgraph_retry(SubgraphRequestRetryAttributes {
        name: endpoint.subgraph_name().to_string(),
        aborted,
    });
}

pub(super) fn subgraph_duration<R: Runtime>(
    ctx: ExecutionContext<'_, R>,
    endpoint: GraphqlEndpoint<'_>,
    subgraph_status: SubgraphResponseStatus,
    http_status_code: Option<http::StatusCode>,
    duration: Duration,
) {
    ctx.metrics().record_subgraph_request_duration(
        SubgraphRequestDurationAttributes {
            name: endpoint.subgraph_name().to_string(),
            subgraph_status,
            http_status_code,
        },
        duration,
    );
}

pub(super) fn subgraph_request_size<R: Runtime>(
    ctx: ExecutionContext<'_, R>,
    endpoint: GraphqlEndpoint<'_>,
    size: usize,
) {
    ctx.metrics().record_subgraph_request_size(
        SubgraphRequestBodySizeAttributes {
            name: endpoint.subgraph_name().to_string(),
        },
        size,
    );
}

pub(super) fn subgraph_response_size<R: Runtime>(
    ctx: ExecutionContext<'_, R>,
    endpoint: GraphqlEndpoint<'_>,
    size: usize,
) {
    ctx.metrics().record_subgraph_response_size(
        SubgraphResponseBodySizeAttributes {
            name: endpoint.subgraph_name().to_string(),
        },
        size,
    );
}

pub(super) fn increment_inflight_requests<R: Runtime>(ctx: ExecutionContext<'_, R>, endpoint: GraphqlEndpoint<'_>) {
    ctx.metrics()
        .increment_subgraph_inflight_requests(SubgraphInFlightRequestAttributes {
            name: endpoint.subgraph_name().to_string(),
        });
}

pub(super) fn decrement_inflight_requests<R: Runtime>(ctx: ExecutionContext<'_, R>, endpoint: GraphqlEndpoint<'_>) {
    ctx.metrics()
        .decrement_subgraph_inflight_requests(SubgraphInFlightRequestAttributes {
            name: endpoint.subgraph_name().to_string(),
        });
}

pub(super) fn record_subgraph_cache_hit<R: Runtime>(ctx: ExecutionContext<'_, R>, endpoint: GraphqlEndpoint<'_>) {
    ctx.metrics().record_subgraph_cache_hit(SubgraphCacheHitAttributes {
        name: endpoint.subgraph_name().to_string(),
    });
}

pub(super) fn record_subgraph_cache_miss<R: Runtime>(ctx: ExecutionContext<'_, R>, endpoint: GraphqlEndpoint<'_>) {
    ctx.metrics().record_subgraph_cache_miss(SubgraphCacheMissAttributes {
        name: endpoint.subgraph_name().to_string(),
    });
}
