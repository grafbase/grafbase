mod authorized;
mod pool;
mod responses;
mod subgraph;

use std::{collections::HashMap, sync::Arc, time::SystemTime};

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
    hooks::{AuthorizedHooks, HeaderMap, Hooks, SubgraphHooks},
};
use tracing::{instrument, Span};
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
    gateway: Pool<GatewayComponentInstance>,
    authorization: Pool<AuthorizationComponentInstance>,
    subgraph: Pool<SubgraphComponentInstance>,
    responses: Pool<ResponsesComponentInstance>,
    hook_latencies: Histogram<u64>,
    sender: ChannelLogSender,
}

impl HooksWasiInner {
    pub fn shared_context(&self, context: &Context) -> SharedContext {
        SharedContext::new(Arc::clone(&context.kv), self.sender.clone(), context.trace_id)
    }

    async fn run_and_measure<F, T>(&self, hook_name: &'static str, hook: F) -> Result<T, wasi_component_loader::Error>
    where
        F: Future<Output = Result<T, wasi_component_loader::Error>>,
    {
        let start = SystemTime::now();
        let result = hook.await;
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
        F: Future<Output = Result<Vec<Result<T, GuestError>>, wasi_component_loader::Error>>,
    {
        let start = SystemTime::now();
        let result = hook.await;
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
    pub fn new(loader: Option<ComponentLoader>, meter: &Meter, sender: ChannelLogSender) -> Self {
        match loader.map(Arc::new) {
            Some(loader) => Self(Some(Arc::new(HooksWasiInner {
                gateway: Pool::new(&loader),
                authorization: Pool::new(&loader),
                subgraph: Pool::new(&loader),
                responses: Pool::new(&loader),
                hook_latencies: meter.u64_histogram("grafbase.hook.duration").init(),
                sender,
            }))),
            None => Self(None),
        }
    }
}

impl Hooks for HooksWasi {
    type Context = Context;

    #[instrument(skip_all)]
    async fn on_gateway_request(&self, headers: HeaderMap) -> Result<(Self::Context, HeaderMap), ErrorResponse> {
        let kv = HashMap::new();
        let trace_id = Span::current().context().span().span_context().trace_id();

        let Some(ref inner) = self.0 else {
            return Ok((Context::new(kv, trace_id), headers));
        };

        let mut hook = inner.gateway.get().await;

        inner
            .run_and_measure("on-gateway-request", hook.on_gateway_request(kv, headers))
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

    fn authorized(&self) -> &impl AuthorizedHooks<Self::Context> {
        self
    }

    fn subgraph(&self) -> &impl SubgraphHooks<Self::Context> {
        self
    }

    fn responses(&self) -> &impl runtime::hooks::ResponseHooks<Self::Context> {
        self
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
