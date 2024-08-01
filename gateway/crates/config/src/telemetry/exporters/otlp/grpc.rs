use crate::config::Headers;

use super::ExporterTlsConfig;

/// GRPC exporting configuration
#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OtlpExporterGrpcConfig {
    /// Tls configuration to use on export requests
    pub tls: Option<ExporterTlsConfig>,
    /// Headers to send on export requests
    #[serde(default)]
    pub headers: Headers,
}
