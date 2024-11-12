use super::{default_export_timeout, deserialize_duration, BatchExportConfig};

/// Stdout exporter configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct StdoutExporterConfig {
    /// Enable or disable the exporter
    pub enabled: bool,
    /// Batch export configuration
    pub batch_export: Option<BatchExportConfig>,
    /// The maximum duration to export data.
    /// The default value is 60 seconds.
    #[serde(deserialize_with = "deserialize_duration")]
    pub timeout: chrono::Duration,
}

impl Default for StdoutExporterConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            batch_export: None,
            timeout: default_export_timeout(),
        }
    }
}
