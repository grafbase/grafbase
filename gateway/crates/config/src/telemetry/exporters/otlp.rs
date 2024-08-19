mod headers;

pub use headers::Headers;

use super::{default_export_timeout, deserialize_duration, BatchExportConfig};
use std::{path::PathBuf, str::FromStr};
use url::Url;

/// Otlp exporter configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OtlpExporterConfig {
    /// Enable or disable the exporter
    pub enabled: bool,
    /// Endpoint of the otlp collector
    pub endpoint: Url,
    /// Batch export configuration
    pub batch_export: BatchExportConfig,
    /// Protocol to use when exporting
    pub protocol: OtlpExporterProtocol,
    /// GRPC exporting configuration
    pub grpc: Option<OtlpExporterGrpcConfig>,
    /// HTTP exporting configuration
    pub http: Option<OtlpExporterHttpConfig>,
    /// The maximum duration to export data.
    /// The default value is 60 seconds.
    #[serde(deserialize_with = "deserialize_duration")]
    pub timeout: chrono::Duration,
}

impl Default for OtlpExporterConfig {
    fn default() -> Self {
        Self {
            endpoint: Url::from_str("http://127.0.0.1:4317").unwrap(),
            enabled: false,
            batch_export: Default::default(),
            protocol: Default::default(),
            grpc: None,
            http: None,
            timeout: default_export_timeout(),
        }
    }
}

/// OTLP Exporter protocol
#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OtlpExporterProtocol {
    /// GRPC protocol
    #[default]
    Grpc,
    /// HTTP protocol
    Http,
}

/// GRPC exporting configuration
#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OtlpExporterGrpcConfig {
    /// Tls configuration to use on export requests
    pub tls: Option<OtlpExporterTlsConfig>,
    /// Headers to send on export requests
    pub headers: Headers,
}

/// OTLP HTTP exporting configuration
#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OtlpExporterHttpConfig {
    /// Http headers to send on export requests
    pub headers: Headers,
}

/// OTLP GRPC TLS export configuration
#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
/// Wraps tls configuration used when exporting data.
/// Any files referenced are read in *sync* fashion using `[std::fs::read]`.
pub struct OtlpExporterTlsConfig {
    /// Domain name against which to verify the server's TLS certificate
    pub domain_name: Option<String>,
    /// Path to the key of the `cert`
    pub key: Option<PathBuf>,
    /// Path to the X509 Certificate file, in pem format, that represents the client identity to present to the server.
    pub cert: Option<PathBuf>,
    /// Path to the X509 CA Certificate file, in pem format, against which to verify the server's TLS certificate.
    pub ca: Option<PathBuf>,
}

#[cfg(feature = "otlp")]
impl TryFrom<OtlpExporterTlsConfig> for tonic::transport::ClientTlsConfig {
    type Error = std::io::Error;

    fn try_from(value: OtlpExporterTlsConfig) -> Result<tonic::transport::ClientTlsConfig, Self::Error> {
        use std::fs;
        use tonic::transport::{Certificate, ClientTlsConfig, Identity};

        let mut tls = ClientTlsConfig::new();

        if let Some(domain) = value.domain_name {
            tls = tls.domain_name(domain);
        }

        if let Some(ca) = value.ca {
            let ca_cert = fs::read(ca)?;
            tls = tls.ca_certificate(Certificate::from_pem(ca_cert))
        }

        if let Some(cert) = value.cert {
            let cert = fs::read(cert)?;

            let key = value.key.map(fs::read).transpose()?.unwrap_or_default();

            let identity = Identity::from_pem(cert, key);
            tls = tls.identity(identity);
        }

        Ok(tls)
    }
}
