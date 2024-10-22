mod authorized;
mod pool;
mod responses;

use std::{collections::HashMap, sync::Arc, time::SystemTime};

use deadpool::managed::Object;
use futures_util::Future;
use grafbase_telemetry::otel::{
    opentelemetry::{
        metrics::{Histogram, Meter},
        trace::{TraceContextExt, TraceId},
        KeyValue,
    },
    tracing_opentelemetry::OpenTelemetrySpanExt,
};
use pool::Pool;
use runtime::{
    error::{ErrorResponse, PartialErrorCode, PartialGraphqlError},
    hooks::{AuthorizedHooks, HeaderMap, Hooks},
};
use tracing::{info_span, Instrument, Span};
use url::Url;
use wasi_component_loader::ResponsesComponentInstance;
pub use wasi_component_loader::{
    create_log_channel, AccessLogMessage, AuthorizationComponentInstance, ChannelLogReceiver, ChannelLogSender,
    ComponentLoader, Config as HooksWasiConfig, GatewayComponentInstance, GuestError, SharedContext,
    SubgraphComponentInstance,
};

#[derive(Clone)]
pub struct HooksWasi(Option<Arc<HooksWasiInner>>);

#[derive(Clone)]
pub struct Context {
    kv: Arc<HashMap<String, String>>,
    trace_id: TraceId,
}

impl Context {
    pub(crate) fn new(kv: HashMap<String, String>, trace_id: TraceId) -> Self {
        Self {
            kv: Arc::new(kv),
            trace_id,
        }
    }
}

struct HooksWasiInner {
    gateway: Option<Pool<GatewayComponentInstance>>,
    authorization: Option<Pool<AuthorizationComponentInstance>>,
    subgraph: Option<Pool<SubgraphComponentInstance>>,
    responses: Option<Pool<ResponsesComponentInstance>>,
    hook_latencies: Histogram<u64>,
    sender: ChannelLogSender,
}

impl HooksWasiInner {
    pub fn shared_context(&self, context: &Context) -> SharedContext {
        SharedContext::new(Arc::clone(&context.kv), self.sender.clone(), context.trace_id)
    }

    pub async fn get_gateway_instance(
        &self,
        hook_name: &'static str,
    ) -> Option<(Object<pool::ComponentMananger<GatewayComponentInstance>>, Span)> {
        match self.gateway {
            Some(ref pool) => {
                let span = info_span!("hook span", "otel.name" = hook_name);
                let object = pool.get().instrument(span.clone()).await;

                Some((object, span))
            }
            None => None,
        }
    }

    pub async fn get_authorization_instance(
        &self,
        hook_name: &'static str,
    ) -> Option<(Object<pool::ComponentMananger<AuthorizationComponentInstance>>, Span)> {
        match self.authorization {
            Some(ref pool) => {
                let span = info_span!("hook span", "otel.name" = hook_name);
                let object = pool.get().instrument(span.clone()).await;

                Some((object, span))
            }
            None => None,
        }
    }

    pub async fn get_subgraph_instance(
        &self,
        hook_name: &'static str,
    ) -> Option<(Object<pool::ComponentMananger<SubgraphComponentInstance>>, Span)> {
        match self.subgraph {
            Some(ref pool) => {
                let span = info_span!("hook span", "otel.name" = hook_name);
                let object = pool.get().instrument(span.clone()).await;

                Some((object, span))
            }
            None => None,
        }
    }

    pub async fn get_responses_instance(
        &self,
        hook_name: &'static str,
    ) -> Option<(Object<pool::ComponentMananger<ResponsesComponentInstance>>, Span)> {
        match self.responses {
            Some(ref pool) => {
                let span = info_span!("hook span", "otel.name" = hook_name);
                let object = pool.get().instrument(span.clone()).await;

                Some((object, span))
            }
            None => None,
        }
    }

    async fn run_and_measure<F, T>(&self, hook_name: &'static str, hook: F) -> Result<T, wasi_component_loader::Error>
    where
        F: Future<Output = Result<T, wasi_component_loader::Error>> + Instrument,
    {
        let span = info_span!("call instance");
        let start = SystemTime::now();
        let result = hook.instrument(span).await;
        let duration = SystemTime::now().duration_since(start).unwrap_or_default();

        let status = match result {
            Ok(_) => HookStatus::Success,
            Err(wasi_component_loader::Error::Internal(_)) => HookStatus::HostError,
            Err(wasi_component_loader::Error::Guest(_)) => HookStatus::GuestError,
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
            Ok(ref statuses) if statuses.iter().any(|s| s.is_err()) => HookStatus::GuestError,
            Ok(_) => HookStatus::Success,
            Err(wasi_component_loader::Error::Internal(_)) => HookStatus::HostError,
            Err(wasi_component_loader::Error::Guest(_)) => HookStatus::GuestError,
        };

        let attributes = [
            KeyValue::new("grafbase.hook.name", hook_name),
            KeyValue::new("grafbase.hook.status", status.as_str()),
        ];

        self.hook_latencies.record(duration.as_millis() as u64, &attributes);

        result
    }
}

#[derive(Debug, Clone, Copy)]
enum HookStatus {
    Success,
    HostError,
    GuestError,
}

impl HookStatus {
    fn as_str(&self) -> &'static str {
        match self {
            HookStatus::Success => "SUCCESS",
            HookStatus::HostError => "HOST_ERROR",
            HookStatus::GuestError => "GUEST_ERROR",
        }
    }
}

impl HooksWasi {
    pub fn new(
        loader: Option<ComponentLoader>,
        max_pool_size: Option<usize>,
        meter: &Meter,
        sender: ChannelLogSender,
    ) -> Self {
        match loader.map(Arc::new) {
            Some(loader) => Self(Some(Arc::new(HooksWasiInner {
                gateway: Pool::new(&loader, max_pool_size),
                authorization: Pool::new(&loader, max_pool_size),
                subgraph: Pool::new(&loader, max_pool_size),
                responses: Pool::new(&loader, max_pool_size),
                hook_latencies: meter.u64_histogram("grafbase.hook.duration").init(),
                sender,
            }))),
            None => Self(None),
        }
    }
}

impl Hooks for HooksWasi {
    type Context = Context;
    type OnSubgraphResponseOutput = Vec<u8>;
    type OnOperationResponseOutput = Vec<u8>;

    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), ErrorResponse> {
        let kv = HashMap::new();
        let trace_id = Span::current().context().span().span_context().trace_id();

        let Some(ref inner) = self.0 else {
            return Ok((Context::new(kv, trace_id), headers));
        };

        let Some((mut hook, span)) = inner.get_gateway_instance("hook: on-gateway-request").await else {
            return Ok((Context::new(kv, trace_id), headers));
        };

        inner
            .run_and_measure("on-gateway-request", hook.on_gateway_request(kv, headers))
            .instrument(span)
            .await
            .map(|(kv, headers)| (Context::new(kv, trace_id), headers))
            .map_err(|err| match err {
                wasi_component_loader::Error::Internal(err) => {
                    tracing::error!("on_gateway_request error: {err}");
                    ErrorResponse::from(PartialGraphqlError::internal_hook_error())
                }
                wasi_component_loader::Error::Guest(err) => {
                    guest_error_as_gql(err, PartialErrorCode::BadRequest).into()
                }
            })
    }

    async fn on_subgraph_request(
        &self,
        context: &Context,
        subgraph_name: &str,
        method: http::Method,
        url: &Url,
        headers: HeaderMap,
    ) -> Result<HeaderMap, PartialGraphqlError> {
        let Some(ref inner) = self.0 else {
            return Ok(headers);
        };

        let Some((mut hook, span)) = inner.get_subgraph_instance("hook: on-subgraph-request").await else {
            return Ok(headers);
        };

        inner
            .run_and_measure(
                "on-subgraph-request",
                hook.on_subgraph_request(inner.shared_context(context), subgraph_name, method, url, headers),
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

fn guest_error_as_gql(error: wasi_component_loader::GuestError, code: PartialErrorCode) -> PartialGraphqlError {
    let extensions = error
        .extensions
        .into_iter()
        .map(|(key, value)| {
            let value = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));

            (key.into(), value)
        })
        .collect();

    PartialGraphqlError {
        message: error.message.into(),
        code,
        extensions,
    }
}
