/// Contains opentelemetry tracing integrations, namely [tracing_subscriber::Layer]'s and
pub mod layer;

// re-exporting otel libs
pub use opentelemetry;
pub use opentelemetry_sdk;
pub use tracing_opentelemetry;
