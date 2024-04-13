use std::sync::Arc;
use tokio::sync::watch;

use grafbase_tracing::otel::opentelemetry_sdk::trace::TracerProvider;

use super::gateway::EngineWatcher;

struct ServerStateInner {
    gateway: EngineWatcher,
    tracer_provider: Option<watch::Receiver<TracerProvider>>,
}

#[derive(Clone)]
pub(super) struct ServerState {
    inner: Arc<ServerStateInner>,
}

impl ServerState {
    pub(super) fn new(gateway: EngineWatcher, tracer_provider: Option<watch::Receiver<TracerProvider>>) -> Self {
        Self {
            inner: Arc::new(ServerStateInner {
                gateway,
                tracer_provider,
            }),
        }
    }

    pub(crate) fn gateway(&self) -> &EngineWatcher {
        &self.inner.gateway
    }

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
