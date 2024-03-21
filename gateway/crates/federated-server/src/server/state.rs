use std::sync::Arc;

use grafbase_tracing::otel::opentelemetry_sdk::trace::TracerProvider;

use super::gateway::GatewayWatcher;

struct ServerStateInner {
    gateway: GatewayWatcher,
    tracer_provider: Option<TracerProvider>,
}

#[derive(Clone)]
pub(super) struct ServerState {
    inner: Arc<ServerStateInner>,
}

impl ServerState {
    pub(super) fn new(gateway: GatewayWatcher, tracer_provider: Option<TracerProvider>) -> Self {
        Self {
            inner: Arc::new(ServerStateInner {
                gateway,
                tracer_provider,
            }),
        }
    }

    pub(crate) fn gateway(&self) -> &GatewayWatcher {
        &self.inner.gateway
    }

    pub(crate) fn tracer_provider(&self) -> Option<&TracerProvider> {
        self.inner.tracer_provider.as_ref()
    }
}
