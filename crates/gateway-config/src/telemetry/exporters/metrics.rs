use super::OpenTelemetryExportersConfig;

/// Metrics configuration
#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MetricsConfig {
    /// Exporters configurations
    pub exporters: OpenTelemetryExportersConfig,
    /// Prometheus exporter configuration
    pub prometheus: Option<PrometheusConfig>,
}

/// Prometheus exporter configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PrometheusConfig {
    /// Enable the Prometheus metrics exporter
    pub enabled: bool,
    /// Address for the Prometheus metrics HTTP server
    pub listen_address: Option<std::net::SocketAddr>,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            listen_address: None,
        }
    }
}
