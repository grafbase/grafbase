use runtime::{error::PartialGraphqlError, hooks::ResponseHooks};
use tracing::Instrument;
use wasi_component_loader::{
    CacheStatus, ExecutedHttpRequest, ExecutedOperation, ExecutedSubgraphRequest, FieldError, GraphqlResponseStatus,
    RequestError, SubgraphRequestExecutionKind, SubgraphResponse,
};

use crate::HooksWasi;

use super::Context;

impl ResponseHooks<Context> for HooksWasi {
    async fn on_subgraph_response(
        &self,
        context: &Context,
        request: runtime::hooks::ExecutedSubgraphRequest<'_>,
    ) -> Result<Vec<u8>, PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Ok(Vec::new());
        };

        let Some((mut hook, span)) = inner.get_responses_instance("hook: on-subgraph-response").await else {
            return Ok(Vec::new());
        };

        let runtime::hooks::ExecutedSubgraphRequest {
            subgraph_name,
            method,
            url,
            executions,
            cache_status,
            total_duration,
            has_graphql_errors,
        } = request;

        let request = ExecutedSubgraphRequest {
            subgraph_name: subgraph_name.to_string(),
            method: method.to_string(),
            url: url.to_string(),
            executions: executions
                .into_iter()
                .map(|execution| match execution {
                    runtime::hooks::SubgraphRequestExecutionKind::InternalServerError => {
                        SubgraphRequestExecutionKind::InternalServerError
                    }
                    runtime::hooks::SubgraphRequestExecutionKind::HookError => SubgraphRequestExecutionKind::HookError,
                    runtime::hooks::SubgraphRequestExecutionKind::RequestError => {
                        SubgraphRequestExecutionKind::RequestError
                    }
                    runtime::hooks::SubgraphRequestExecutionKind::RateLimited => {
                        SubgraphRequestExecutionKind::RateLimited
                    }
                    runtime::hooks::SubgraphRequestExecutionKind::Responsed(info) => {
                        SubgraphRequestExecutionKind::Response(SubgraphResponse {
                            connection_time_ms: info.connection_time_ms,
                            response_time_ms: info.response_time_ms,
                            status_code: info.status_code,
                        })
                    }
                })
                .collect(),
            cache_status: match cache_status {
                runtime::hooks::CacheStatus::Hit => CacheStatus::Hit,
                runtime::hooks::CacheStatus::PartialHit => CacheStatus::PartialHit,
                runtime::hooks::CacheStatus::Miss => CacheStatus::Miss,
            },
            total_duration_ms: total_duration.as_millis() as u64,
            has_errors: has_graphql_errors,
        };

        inner
            .run_and_measure(
                "on-subgraph-response",
                hook.on_subgraph_response(inner.shared_context(context), request),
            )
            .instrument(span)
            .await
            .map_err(|err| {
                tracing::error!("on_subgraph_response error: {err}");
                PartialGraphqlError::internal_hook_error()
            })
    }

    async fn on_operation_response(
        &self,
        context: &Context,
        operation: runtime::hooks::ExecutedOperation<'_>,
    ) -> Result<Vec<u8>, PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Ok(Vec::new());
        };

        let Some((mut hook, span)) = inner.get_responses_instance("hook: on-operation-response").await else {
            return Ok(Vec::new());
        };

        let runtime::hooks::ExecutedOperation {
            duration,
            status,
            on_subgraph_response_outputs,
            name,
            document,
            prepare_duration,
            cached_plan,
        } = operation;

        let operation = ExecutedOperation {
            duration_ms: duration.as_millis() as u64,
            status: match status {
                grafbase_telemetry::graphql::GraphqlResponseStatus::Success => GraphqlResponseStatus::Success,
                grafbase_telemetry::graphql::GraphqlResponseStatus::FieldError { count, data_is_null } => {
                    GraphqlResponseStatus::FieldError(FieldError { count, data_is_null })
                }
                grafbase_telemetry::graphql::GraphqlResponseStatus::RequestError { count } => {
                    GraphqlResponseStatus::RequestError(RequestError { count })
                }
                grafbase_telemetry::graphql::GraphqlResponseStatus::RefusedRequest => {
                    GraphqlResponseStatus::RefusedRequest
                }
            },
            on_subgraph_response_outputs,
            name: name.map(str::to_string),
            document: document.to_string(),
            prepare_duration_ms: prepare_duration.as_millis() as u64,
            cached_plan,
        };

        inner
            .run_and_measure(
                "on-operation-response",
                hook.on_operation_response(inner.shared_context(context), operation),
            )
            .instrument(span)
            .await
            .map_err(|err| {
                tracing::error!("on_gateway_response error: {err}");
                PartialGraphqlError::internal_hook_error()
            })
    }

    async fn on_http_response(
        &self,
        context: &Context,
        request: runtime::hooks::ExecutedHttpRequest,
    ) -> Result<(), PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Ok(());
        };

        let Some((mut hook, span)) = inner.get_responses_instance("hook: on-http-response").await else {
            return Ok(());
        };

        let runtime::hooks::ExecutedHttpRequest {
            method,
            url,
            status_code,
            on_operation_response_outputs,
        } = request;

        let request = ExecutedHttpRequest {
            method: method.to_string(),
            url: url.to_string(),
            status_code: status_code.as_u16(),
            on_operation_response_outputs,
        };

        inner
            .run_and_measure(
                "on-http-response",
                hook.on_http_response(inner.shared_context(context), request),
            )
            .instrument(span)
            .await
            .map_err(|err| {
                tracing::error!("on_http_response error: {err}");
                PartialGraphqlError::internal_hook_error()
            })
    }
}
