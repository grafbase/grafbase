use grafbase_tracing::otel::opentelemetry_sdk::trace::TracerProvider;
use ulid::Ulid;

/// Holds legos to deal with opentelemetry tracing
pub struct OtelTracing {
    /// The tracer provider attached to the otel layer
    /// It's a receiver with an optional because:
    ///     - the layer related to the handler might be a noop_layer and therefore has no provider attached
    ///     - it can be replaced on reload, and we want the latest
    pub tracer_provider: tokio::sync::watch::Receiver<TracerProvider>,
    /// A channel to trigger the otel layer reload with new data
    pub reload_trigger: tokio::sync::oneshot::Sender<OtelReload>,
}

/// Payload sent when triggering an otel layer reload
#[derive(Debug, Default)]
pub struct OtelReload {
    /// Graph id
    pub graph_id: Ulid,
    /// Branch id
    pub branch_id: Ulid,
    /// Branch name
    pub branch_name: String,
}
