use runtime::{
    error::{ErrorResponse, PartialErrorCode, PartialGraphqlError},
    hooks::ResponseHooks,
};
use wasi_component_loader::{
    CacheStatus, ExecutedGatewayRequest, ExecutedHttpRequest, ExecutedSubgraphRequest, FieldError,
    GraphqlResponseStatus, Operation, RequestError, SubgraphResponseInfo,
};

use crate::HooksWasi;

use super::{guest_error_as_gql, Context};

impl ResponseHooks<Context> for HooksWasi {
    async fn on_subgraph_response(
        &self,
        context: &Context,
        request: runtime::hooks::ExecutedSubgraphRequest<'_>,
    ) -> Result<Vec<u8>, ErrorResponse> {
        let Some(ref inner) = self.0 else {
            return Ok(Vec::new());
        };

        let mut hook = inner.responses.get().await;

        let runtime::hooks::ExecutedSubgraphRequest {
            subgraph_name,
            method,
            url,
            response_infos,
            cache_status,
            total_duration,
            has_errors,
        } = request;

        let request = ExecutedSubgraphRequest {
            subgraph_name: subgraph_name.to_string(),
            method: method.to_string(),
            url: url.to_string(),
            response_infos: response_infos
                .into_iter()
                .map(|info| SubgraphResponseInfo {
                    connection_time: info.connection_time,
                    response_time: info.response_time,
                    status_code: info.status_code,
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
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(err) => {
                    tracing::error!("on_subgraph_response error: {err}");
                    PartialGraphqlError::internal_hook_error().into()
                }
                wasi_component_loader::Error::Guest(err) => guest_error_as_gql(err, PartialErrorCode::HookError).into(),
            })
    }

    async fn on_gateway_response(
        &self,
        context: &Context,
        operation: runtime::hooks::Operation<'_>,
        request: runtime::hooks::ExecutedGatewayRequest,
    ) -> Result<Vec<u8>, ErrorResponse> {
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

        let runtime::hooks::ExecutedGatewayRequest {
            duration,
            status,
            on_subgraph_request_outputs,
        } = request;

        let operation = Operation {
            name: name.map(ToString::to_string),
            document: document.to_string(),
            prepare_duration,
            cached,
        };

        let request = ExecutedGatewayRequest {
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
                "on-gateway-response",
                hook.on_gateway_response(inner.shared_context(context), operation, request),
            )
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(err) => {
                    tracing::error!("on_gateway_response error: {err}");
                    PartialGraphqlError::internal_hook_error().into()
                }
                wasi_component_loader::Error::Guest(err) => guest_error_as_gql(err, PartialErrorCode::HookError).into(),
            })
    }

    async fn on_http_response(
        &self,
        context: &Context,
        request: runtime::hooks::ExecutedHttpRequest<'_>,
    ) -> Result<(), ErrorResponse> {
        let Some(ref inner) = self.0 else {
            return Ok(());
        };

        let mut hook = inner.responses.get().await;

        let runtime::hooks::ExecutedHttpRequest {
            method,
            url,
            status_code,
            on_gateway_response_outputs,
        } = request;

        let request = ExecutedHttpRequest {
            method: method.to_string(),
            url: url.to_string(),
            status_code: status_code.as_u16(),
            on_gateway_response_outputs,
        };

        inner
            .run_and_measure(
                "on-http-response",
                hook.on_http_response(inner.shared_context(context), request),
            )
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(err) => {
                    tracing::error!("on_http_response error: {err}");
                    PartialGraphqlError::internal_hook_error().into()
                }
                wasi_component_loader::Error::Guest(err) => guest_error_as_gql(err, PartialErrorCode::HookError).into(),
            })
    }
}
