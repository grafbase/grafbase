/// exporter
#[cfg(feature = "otlp")]
pub mod exporter;
/// Contains opentelemetry tracing integrations, namely [tracing_subscriber::Layer]'s and
pub mod layer;
/// logs related otel functions
pub mod logs;
/// metrics related otel functions
pub mod metrics;
/// For creation of a tracing provider.
pub mod traces;

// re-exporting otel libs
pub use opentelemetry;
pub use opentelemetry_appender_tracing;
pub use opentelemetry_sdk;
pub use tracing_opentelemetry;
pub use tracing_subscriber;
