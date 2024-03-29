use grafbase_tracing::config::TracingConfig;

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryConfig {
    /// The name of the service
    pub service_name: String,
    /// Tracing config
    #[serde(default)]
    pub tracing: TracingConfig,
}
