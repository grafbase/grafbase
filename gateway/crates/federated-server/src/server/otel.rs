use grafbase_telemetry::otel::opentelemetry_sdk::trace::TracerProvider;
use ulid::Ulid;

/// Holds legos to deal with opentelemetry tracing
pub struct OtelTracing {
    /// The tracer provider attached to the otel layer
    /// It's a receiver with an optional because:
    ///     - the layer related to the handler might be a noop_layer and therefore has no provider attached
    ///     - it can be replaced on reload, and we want the latest
    pub tracer_provider: tokio::sync::watch::Receiver<TracerProvider>,
    /// A channel to trigger the otel layer reload with new data. While it's a mpsc, only the first
    /// reload will be taken into account.
    pub reload_trigger: tokio::sync::oneshot::Sender<OtelReload>,
    /// A channel to receive confirmation that the OTEL reload happened.
    pub reload_ack_receiver: tokio::sync::oneshot::Receiver<()>,
}

/// Payload sent when triggering an otel layer reload
#[derive(Debug)]
pub struct OtelReload {
    /// Graph id
    pub graph_id: Ulid,
    /// Branch id
    pub branch_id: Ulid,
    /// Branch name
    pub branch_name: String,
}
