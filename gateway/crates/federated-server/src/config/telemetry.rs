use std::collections::HashMap;

use grafbase_tracing::config::TracingConfig;

/// Holds telemetry configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryConfig {
    /// The name of the service
    pub service_name: String,
    /// Additional resource attributes
    #[serde(default)]
    pub resource_attributes: HashMap<String, String>,
    /// Tracing config
    #[serde(default)]
    pub tracing: TracingConfig,
}
