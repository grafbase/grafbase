use super::{default_filter, ExportersConfig};

/// Logs configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogsConfig {
    /// Filter to be applied
    #[serde(default = "default_filter")]
    pub filter: String,
    /// Exporters configurations
    #[serde(default)]
    pub exporters: ExportersConfig,
}
