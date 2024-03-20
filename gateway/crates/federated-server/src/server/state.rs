use std::sync::Arc;

use crate::config::TelemetryConfig;

use super::gateway::GatewayWatcher;

struct ServerStateInner {
    gateway: GatewayWatcher,
    telemetry_config: Option<TelemetryConfig>,
}

#[derive(Clone)]
pub(super) struct ServerState {
    inner: Arc<ServerStateInner>,
}

impl ServerState {
    pub(super) fn new(gateway: GatewayWatcher, telemetry_config: Option<TelemetryConfig>) -> Self {
        Self {
            inner: Arc::new(ServerStateInner {
                gateway,
                telemetry_config,
            }),
        }
    }

    pub(crate) fn gateway(&self) -> &GatewayWatcher {
        &self.inner.gateway
    }

    pub(crate) fn telemetry_config(&self) -> Option<&TelemetryConfig> {
        self.inner.telemetry_config.as_ref()
    }
}
