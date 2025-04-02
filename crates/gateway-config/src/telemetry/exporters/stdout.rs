use std::time::Duration;

use super::{BatchExportConfig, default_export_timeout};

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
    #[serde(deserialize_with = "duration_str::deserialize_duration")]
    pub timeout: Duration,
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
