use super::ExportersConfig;

/// Logs configuration
#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LogsConfig {
    /// Exporters configurations
    pub exporters: ExportersConfig,
}
