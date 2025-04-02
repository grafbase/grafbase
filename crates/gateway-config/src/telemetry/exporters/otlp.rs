mod headers;

pub use headers::Headers;

use super::{BatchExportConfig, default_export_timeout};
use std::{path::PathBuf, time::Duration};
use url::Url;

// FIXME: Please make me disappear me, so horrible.
#[derive(Debug)]
pub struct LayeredOtlExporterConfig {
    pub global: OtlpExporterConfig,
    pub local: OtlpExporterConfig,
}

impl LayeredOtlExporterConfig {
    pub fn is_enabled(&self) -> bool {
        self.local.enabled.or(self.global.enabled).unwrap_or_default()
    }

    pub fn timeout(&self) -> Duration {
        self.local
            .timeout
            .or(self.global.timeout)
            .unwrap_or_else(default_export_timeout)
    }

    pub fn batch_export(&self) -> BatchExportConfig {
        self.local.batch_export.or(self.global.batch_export).unwrap_or_default()
    }

    pub fn protocol(&self) -> OtlpExporterProtocolConfig {
        match self.local.protocol.or(self.global.protocol).unwrap_or_default() {
            OtlpExporterProtocol::Grpc => OtlpExporterProtocolConfig::Grpc(
                self.local.grpc.clone().or(self.global.grpc.clone()).unwrap_or_default(),
            ),
            OtlpExporterProtocol::Http => OtlpExporterProtocolConfig::Http(
                self.local.http.clone().or(self.global.http.clone()).unwrap_or_default(),
            ),
        }
    }
}

pub enum OtlpExporterProtocolConfig {
    Grpc(OtlpExporterGrpcConfig),
    Http(OtlpExporterHttpConfig),
}

/// Otlp exporter configuration
#[derive(Default, Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OtlpExporterConfig {
    /// Enable or disable the exporter
    pub enabled: Option<bool>,
    /// Endpoint of the otlp collector
    pub endpoint: Option<Url>,
    /// Batch export configuration
    pub batch_export: Option<BatchExportConfig>,
    /// Protocol to use when exporting
    pub protocol: Option<OtlpExporterProtocol>,
    /// GRPC exporting configuration
    pub grpc: Option<OtlpExporterGrpcConfig>,
    /// HTTP exporting configuration
    pub http: Option<OtlpExporterHttpConfig>,
    /// The maximum duration to export data.
    /// The default value is 60 seconds.
    #[serde(deserialize_with = "duration_str::deserialize_option_duration")]
    pub timeout: Option<std::time::Duration>,
}

/// OTLP Exporter protocol
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Deserialize)]
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
