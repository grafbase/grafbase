use super::{default_export_timeout, deserialize_duration, BatchExportConfig};

/// Stdout exporter configuration
#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StdoutExporterConfig {
    /// Enable or disable the exporter
    #[serde(default)]
    pub enabled: bool,
    /// Batch export configuration
    #[serde(default)]
    pub batch_export: Option<BatchExportConfig>,
    /// The maximum duration to export data.
    /// The default value is 60 seconds.
    #[serde(deserialize_with = "deserialize_duration", default = "default_export_timeout")]
    pub timeout: chrono::Duration,
}
