/// Contains opentelemetry tracing integrations, namely [tracing_subscriber::Layer]'s and
pub mod layer;
/// For creation of a tracing provider.
pub mod provider;

// re-exporting otel libs
pub use opentelemetry;
pub use opentelemetry_sdk;
pub use tracing_opentelemetry;
pub use tracing_subscriber;
