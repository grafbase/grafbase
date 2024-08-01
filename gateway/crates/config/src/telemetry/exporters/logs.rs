use super::ExportersConfig;

/// Logs configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogsConfig {
    /// Exporters configurations
    #[serde(default)]
    pub exporters: ExportersConfig,
}
