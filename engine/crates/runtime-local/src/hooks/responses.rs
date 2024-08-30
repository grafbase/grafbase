use runtime::{error::PartialGraphqlError, hooks::ResponseHooks};
use wasi_component_loader::{
    CacheStatus, ExecutedHttpRequest, ExecutedOperationRequest, ExecutedSubgraphRequest, FieldError,
    GraphqlResponseStatus, Operation, RequestError, ResponseInfo, ResponseKind,
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

        let mut hook = inner.responses.get().await;

        let runtime::hooks::ExecutedSubgraphRequest {
            subgraph_name,
            method,
            url,
            responses,
            cache_status,
            total_duration,
            has_errors,
        } = request;

        let request = ExecutedSubgraphRequest {
            subgraph_name: subgraph_name.to_string(),
            method: method.to_string(),
            url: url.to_string(),
            responses: responses
                .into_iter()
                .map(|response| match response {
                    runtime::hooks::ResponseKind::SerializationError => ResponseKind::SerializationError,
                    runtime::hooks::ResponseKind::HookError => ResponseKind::HookError,
                    runtime::hooks::ResponseKind::RequestError => ResponseKind::RequestError,
                    runtime::hooks::ResponseKind::RateLimited => ResponseKind::RateLimited,
                    runtime::hooks::ResponseKind::Responsed(info) => ResponseKind::Responsed(ResponseInfo {
                        connection_time: info.connection_time,
                        response_time: info.response_time,
                        status_code: info.status_code,
                    }),
                })
                .collect(),
            cache_status: match cache_status {
                runtime::hooks::CacheStatus::Hit => CacheStatus::Hit,
                runtime::hooks::CacheStatus::PartialHit => CacheStatus::PartialHit,
                runtime::hooks::CacheStatus::Miss => CacheStatus::Miss,
            },
            total_duration,
            has_errors,
        };

        inner
            .run_and_measure(
                "on-subgraph-response",
                hook.on_subgraph_response(inner.shared_context(context), request),
            )
            .await
            .map_err(|err| {
                tracing::error!("on_subgraph_response error: {err}");
                PartialGraphqlError::internal_hook_error()
            })
    }

    async fn on_operation_response(
        &self,
        context: &Context,
        operation: runtime::hooks::Operation<'_>,
        request: runtime::hooks::ExecutedOperationRequest,
    ) -> Result<Vec<u8>, PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Ok(Vec::new());
        };

        let mut hook = inner.responses.get().await;

        let runtime::hooks::Operation {
            name,
            document,
            prepare_duration,
            cached,
        } = operation;

        let runtime::hooks::ExecutedOperationRequest {
            duration,
            status,
            on_subgraph_response_outputs: on_subgraph_request_outputs,
        } = request;

        let operation = Operation {
            name,
            document: document.to_string(),
            prepare_duration,
            cached,
        };

        let request = ExecutedOperationRequest {
            duration,
            status: match status {
                grafbase_telemetry::gql_response_status::GraphqlResponseStatus::Success => {
                    GraphqlResponseStatus::Success
                }
                grafbase_telemetry::gql_response_status::GraphqlResponseStatus::FieldError { count, data_is_null } => {
                    GraphqlResponseStatus::FieldError(FieldError { count, data_is_null })
                }
                grafbase_telemetry::gql_response_status::GraphqlResponseStatus::RequestError { count } => {
                    GraphqlResponseStatus::RequestError(RequestError { count })
                }
                grafbase_telemetry::gql_response_status::GraphqlResponseStatus::RefusedRequest => {
                    GraphqlResponseStatus::RefusedRequest
                }
            },
            on_subgraph_request_outputs,
        };

        inner
            .run_and_measure(
                "on-operation-response",
                hook.on_operation_response(inner.shared_context(context), operation, request),
            )
            .await
            .map_err(|err| {
                tracing::error!("on_gateway_response error: {err}");
                PartialGraphqlError::internal_hook_error()
            })
    }

    async fn on_http_response(
        &self,
        context: &Context,
        request: runtime::hooks::ExecutedHttpRequest<'_>,
    ) -> Result<(), PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Ok(());
        };

        let mut hook = inner.responses.get().await;

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
            .await
            .map_err(|err| {
                tracing::error!("on_http_response error: {err}");
                PartialGraphqlError::internal_hook_error()
            })
    }
}
