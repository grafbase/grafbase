use std::sync::Arc;
use tokio::sync::watch;

use grafbase_telemetry::otel::opentelemetry_sdk::trace::TracerProvider;

use super::gateway::EngineWatcher;

struct ServerStateInner {
    gateway: EngineWatcher,
    tracer_provider: Option<watch::Receiver<TracerProvider>>,
    request_body_limit_bytes: usize,
}

#[derive(Clone)]
pub(super) struct ServerState {
    inner: Arc<ServerStateInner>,
}

impl ServerState {
    pub(super) fn new(
        gateway: EngineWatcher,
        tracer_provider: Option<watch::Receiver<TracerProvider>>,
        request_body_limit_bytes: usize,
    ) -> Self {
        Self {
            inner: Arc::new(ServerStateInner {
                gateway,
                tracer_provider,
                request_body_limit_bytes,
            }),
        }
    }

    pub(crate) fn request_body_limit_bytes(&self) -> usize {
        self.inner.request_body_limit_bytes
    }

    pub(crate) fn gateway(&self) -> &EngineWatcher {
        &self.inner.gateway
    }

    #[allow(unused)] // courtesy of not(lambda) feature flag
    pub(crate) fn tracer_provider(&self) -> Option<TracerProvider> {
        // notes on the clone:
        // - avoid long borrows that could block the producer
        // - tracer provider is backed by an arc so its cheaply cloned
        self.inner
            .tracer_provider
            .as_ref()
            .map(|receiver| receiver.borrow().clone())
    }
}
