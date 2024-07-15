use super::ExportersConfig;

/// Logs configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsConfig {
    /// Exporters configurations
    #[serde(default)]
    pub exporters: ExportersConfig,
}
