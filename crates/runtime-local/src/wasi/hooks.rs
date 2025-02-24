mod authorized;
mod pool;
mod responses;

use std::{collections::HashMap, sync::Arc, time::SystemTime};

use enumflags2::BitFlags;
use futures_util::Future;
use grafbase_telemetry::otel::{
    opentelemetry::{
        KeyValue,
        metrics::{Histogram, Meter},
        trace::TraceContextExt,
    },
    tracing_opentelemetry::OpenTelemetrySpanExt,
};
use pool::Pool;
use runtime::{
    error::{ErrorResponse, PartialErrorCode, PartialGraphqlError},
    hooks::{AuthorizedHooks, HeaderMap, Hooks, SubgraphRequest},
};
use tracing::{Instrument, Span, info_span};
use wasi_component_loader::HookImplementation;
pub use wasi_component_loader::{
    AccessLogMessage, AccessLogReceiver, AccessLogSender, ComponentLoader, GuestError, HooksWasiConfig as Config,
    SharedContext, create_access_log_channel,
};

use super::guest_error_as_gql;

#[derive(Clone)]
pub struct HooksWasi(Option<Arc<HooksWasiInner>>);

struct HooksWasiInner {
    pool: Pool,
    implemented_hooks: BitFlags<HookImplementation>,
    hook_latencies: Histogram<u64>,
}

impl HooksWasiInner {
    async fn run_and_measure<F, T, E>(&self, hook_name: &'static str, hook: F) -> Result<T, E>
    where
        F: Future<Output = Result<T, E>> + Instrument,
        E: HookError,
    {
        let span = info_span!("call instance");
        let start = SystemTime::now();
        let result = hook.instrument(span).await;
        let duration = SystemTime::now().duration_since(start).unwrap_or_default();

        let status = match result {
            Ok(_) => HookStatus::Success,
            Err(ref error) => HookStatus::Error(error.status()),
        };

        let attributes = [
            KeyValue::new("grafbase.hook.name", hook_name),
            KeyValue::new("grafbase.hook.status", status.as_str()),
        ];

        self.hook_latencies.record(duration.as_millis() as u64, &attributes);

        result
    }

    async fn run_and_measure_multi_error<F, T>(
        &self,
        hook_name: &'static str,
        hook: F,
    ) -> Result<Vec<Result<T, GuestError>>, wasi_component_loader::Error>
    where
        F: Future<Output = Result<Vec<Result<T, GuestError>>, wasi_component_loader::Error>> + Instrument,
    {
        let span = info_span!("call instance");
        let start = SystemTime::now();
        let result = hook.instrument(span).await;
        let duration = SystemTime::now().duration_since(start).unwrap_or_default();

        let status = match result {
            Ok(ref statuses) if statuses.iter().any(|s| s.is_err()) => HookStatus::Error(ErrorStatus::GuestError),
            Ok(_) => HookStatus::Success,
            Err(wasi_component_loader::Error::Internal(_)) => HookStatus::Error(ErrorStatus::HostError),
            Err(wasi_component_loader::Error::Guest(_)) => HookStatus::Error(ErrorStatus::GuestError),
        };

        let attributes = [
            KeyValue::new("grafbase.hook.name", hook_name),
            KeyValue::new("grafbase.hook.status", status.as_str()),
        ];

        self.hook_latencies.record(duration.as_millis() as u64, &attributes);

        result
    }
}

trait HookError {
    fn status(&self) -> ErrorStatus;
}

impl HookError for wasi_component_loader::Error {
    fn status(&self) -> ErrorStatus {
        match self {
            wasi_component_loader::Error::Internal(_) => ErrorStatus::HostError,
            wasi_component_loader::Error::Guest(_) => ErrorStatus::GuestError,
        }
    }
}

impl HookError for wasi_component_loader::GatewayError {
    fn status(&self) -> ErrorStatus {
        match self {
            wasi_component_loader::GatewayError::Internal(_) => ErrorStatus::HostError,
            wasi_component_loader::GatewayError::Guest(_) => ErrorStatus::GuestError,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum HookStatus {
    Success,
    Error(ErrorStatus),
}

#[derive(Debug, Clone, Copy)]
enum ErrorStatus {
    HostError,
    GuestError,
}

impl HookStatus {
    fn as_str(&self) -> &'static str {
        match self {
            HookStatus::Success => "SUCCESS",
            HookStatus::Error(ErrorStatus::HostError) => "HOST_ERROR",
            HookStatus::Error(ErrorStatus::GuestError) => "GUEST_ERROR",
        }
    }
}

impl HooksWasi {
    pub async fn new(
        loader: Option<ComponentLoader>,
        max_pool_size: Option<usize>,
        meter: &Meter,
        access_log: AccessLogSender,
    ) -> Self {
        match loader.map(Arc::new) {
            Some(loader) => {
                let pool = Pool::new(&loader, max_pool_size, access_log);
                let instance = pool.get().await;
                let implemented_hooks = instance.hooks_implemented();

                let inner = HooksWasiInner {
                    pool,
                    implemented_hooks,
                    hook_latencies: meter.u64_histogram("grafbase.hook.duration").build(),
                };

                Self(Some(Arc::new(inner)))
            }
            None => Self(None),
        }
    }
}

impl Hooks for HooksWasi {
    type Context = SharedContext;
    type OnSubgraphResponseOutput = Vec<u8>;
    type OnOperationResponseOutput = Vec<u8>;

    fn new_context(&self) -> Self::Context {
        let kv = HashMap::new();
        let trace_id = Span::current().context().span().span_context().trace_id();
        SharedContext::new(Arc::new(kv), trace_id)
    }

    async fn on_gateway_request(
        &self,
        url: &str,
        headers: HeaderMap,
    ) -> Result<(Self::Context, HeaderMap), (Self::Context, ErrorResponse)> {
        let kv = HashMap::new();
        let trace_id = Span::current().context().span().span_context().trace_id();

        let Some(ref inner) = self.0 else {
            return Ok((SharedContext::new(Arc::new(kv), trace_id), headers));
        };

        if !inner.implemented_hooks.contains(HookImplementation::OnGatewayRequest) {
            return Ok((SharedContext::new(Arc::new(kv), trace_id), headers));
        }

        let span = info_span!("hook: on-gateway-request");
        let mut hook = inner.pool.get().instrument(span.clone()).await;

        inner
            .run_and_measure("on-gateway-request", hook.on_gateway_request(kv, url, headers))
            .instrument(span)
            .await
            .map(|(kv, headers)| (SharedContext::new(Arc::new(kv), trace_id), headers))
            .map_err(|err| {
                let context = SharedContext::new(Arc::new(HashMap::new()), trace_id);

                match err {
                    wasi_component_loader::GatewayError::Internal(err) => {
                        tracing::error!("on_gateway_request error: {err}");

                        let response = ErrorResponse {
                            status: http::StatusCode::INTERNAL_SERVER_ERROR,
                            errors: vec![PartialGraphqlError::internal_hook_error()],
                        };

                        (context, response)
                    }
                    wasi_component_loader::GatewayError::Guest(error) => {
                        let status = http::StatusCode::from_u16(error.status_code)
                            .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);

                        let errors = error
                            .errors
                            .into_iter()
                            .map(|error| guest_error_as_gql(error, PartialErrorCode::BadRequest))
                            .collect();

                        let response = ErrorResponse { status, errors };

                        (context, response)
                    }
                }
            })
    }

    async fn on_subgraph_request(
        &self,
        context: &Self::Context,
        subgraph_name: &str,
        request: SubgraphRequest,
    ) -> Result<SubgraphRequest, PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Ok(request);
        };

        if !inner.implemented_hooks.contains(HookImplementation::OnSubgraphRequest) {
            return Ok(request);
        }

        let span = info_span!("hook: on-subgraph-request");
        let mut hook = inner.pool.get().instrument(span.clone()).await;

        inner
            .run_and_measure(
                "on-subgraph-request",
                hook.on_subgraph_request(context.clone(), subgraph_name, request),
            )
            .instrument(span)
            .await
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(err) => {
                    tracing::error!("on_gateway_request error: {err}");
                    PartialGraphqlError::internal_hook_error()
                }
                wasi_component_loader::Error::Guest(err) => guest_error_as_gql(err, PartialErrorCode::HookError),
            })
    }

    fn authorized(&self) -> &impl AuthorizedHooks<Self::Context> {
        self
    }

    fn on_subgraph_response(
        &self,
        context: &Self::Context,
        request: runtime::hooks::ExecutedSubgraphRequest<'_>,
    ) -> impl Future<Output = Result<Self::OnSubgraphResponseOutput, PartialGraphqlError>> + Send {
        HooksWasi::on_subgraph_response(self, context, request)
    }

    fn on_operation_response(
        &self,
        context: &Self::Context,
        operation: runtime::hooks::ExecutedOperation<'_, Self::OnSubgraphResponseOutput>,
    ) -> impl Future<Output = Result<Self::OnOperationResponseOutput, PartialGraphqlError>> + Send {
        HooksWasi::on_operation_response(self, context, operation)
    }

    fn on_http_response(
        &self,
        context: &Self::Context,
        request: runtime::hooks::ExecutedHttpRequest<Self::OnOperationResponseOutput>,
    ) -> impl Future<Output = Result<(), PartialGraphqlError>> + Send {
        HooksWasi::on_http_response(self, context, request)
    }
}
