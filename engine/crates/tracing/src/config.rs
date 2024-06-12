use std::collections::HashMap;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::str::FromStr;
use std::usize;

use http::{HeaderName, HeaderValue};
use serde::de::{Error as DeserializeError, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
#[cfg(feature = "otlp")]
use tonic::transport::{Certificate, ClientTlsConfig, Identity};
use url::Url;

use crate::error::TracingError;

pub(crate) const DEFAULT_COLLECT_VALUE: usize = 128;
/// Default tracing filter to be applied on spans that are client facing
pub const DEFAULT_FILTER: &str = "grafbase-gateway=info,federated-server=info,grafbase=info,off";
pub(crate) const DEFAULT_SAMPLING: f64 = 0.15;
// FIXME: Use this constant when `unwrap()` becomes const-stable.
// const DEFAULT_EXPORT_TIMEOUT: chrono::Duration = chrono::Duration::try_seconds(60).unwrap();

/// Tracing configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TracingConfig {
    /// If tracing should be enabled or not
    #[serde(default)]
    pub enabled: bool,
    /// Filter to be applied
    #[serde(default = "default_filter")]
    pub filter: String,
    /// The sampler between 0.0 and 1.0.
    /// Default is 0.15.
    #[serde(default = "default_sampling", deserialize_with = "deserialize_sampling")]
    pub sampling: f64,
    /// Collection configuration
    #[serde(default)]
    pub collect: TracingCollectConfig,
    /// Exporting configuration for batched operations
    #[serde(default)]
    pub batch_export: TracingBatchExportConfig,
    /// Exporters configurations
    #[serde(default)]
    pub exporters: TracingExportersConfig,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: Default::default(),
            filter: DEFAULT_FILTER.to_string(),
            sampling: DEFAULT_SAMPLING,
            collect: Default::default(),
            batch_export: Default::default(),
            exporters: Default::default(),
        }
    }
}

fn deserialize_sampling<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let input = f64::deserialize(deserializer)?;

    if !(0.0..=1.0).contains(&input) {
        return Err(DeserializeError::custom("input value should be 0..1"));
    }

    Ok(input)
}

fn default_sampling() -> f64 {
    DEFAULT_SAMPLING
}

fn default_filter() -> String {
    DEFAULT_FILTER.to_string()
}

/// Tracing collection configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TracingCollectConfig {
    /// The maximum events per span before discarding.
    /// The default is 128.
    #[serde(default = "default_collect")]
    pub max_events_per_span: usize,
    /// The maximum attributes per span before discarding.
    /// The default is 128.
    #[serde(default = "default_collect")]
    pub max_attributes_per_span: usize,
    /// The maximum links per span before discarding.
    /// The default is 128.
    #[serde(default = "default_collect")]
    pub max_links_per_span: usize,
    /// The maximum attributes per event before discarding.
    /// The default is 128.
    #[serde(default = "default_collect")]
    pub max_attributes_per_event: usize,
    /// The maximum attributes per link before discarding.
    /// The default is 128.
    #[serde(default = "default_collect")]
    pub max_attributes_per_link: usize,
}

impl Default for TracingCollectConfig {
    fn default() -> Self {
        Self {
            max_events_per_span: 128,
            max_attributes_per_span: 128,
            max_links_per_span: 128,
            max_attributes_per_event: 128,
            max_attributes_per_link: 128,
        }
    }
}

fn default_collect() -> usize {
    DEFAULT_COLLECT_VALUE
}

/// Configuration for batched exports
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TracingBatchExportConfig {
    /// The delay, in seconds, between two consecutive processing of batches.
    /// The default value is 5 seconds.
    #[serde(
        deserialize_with = "deserialize_duration",
        default = "TracingBatchExportConfig::default_scheduled_delay"
    )]
    pub scheduled_delay: chrono::Duration,

    /// The maximum queue size to buffer spans for delayed processing. If the
    /// queue gets full it drops the spans.
    /// The default value of is 2048.
    #[serde(default = "TracingBatchExportConfig::default_max_queue_size")]
    pub max_queue_size: usize,

    /// The maximum number of spans to process in a single batch. If there are
    /// more than one batch worth of spans then it processes multiple batches
    /// of spans one batch after the other without any delay.
    /// The default value is 512.
    #[serde(default = "TracingBatchExportConfig::default_max_export_batch_size")]
    pub max_export_batch_size: usize,

    /// Maximum number of concurrent exports
    ///
    /// Limits the number of spawned tasks for exports and thus resources consumed
    /// by an exporter. A value of 1 will cause exports to be performed
    /// synchronously on the [`BatchSpanProcessor`] task.
    /// The default is 1.
    #[serde(default = "TracingBatchExportConfig::default_max_concurrent_exports")]
    pub max_concurrent_exports: usize,
}

impl TracingBatchExportConfig {
    fn default_scheduled_delay() -> chrono::Duration {
        chrono::Duration::try_seconds(5).expect("must be fine")
    }
    fn default_max_queue_size() -> usize {
        2048
    }
    fn default_max_export_batch_size() -> usize {
        512
    }
    fn default_max_concurrent_exports() -> usize {
        1
    }
}

fn deserialize_duration<'de, D>(deserializer: D) -> Result<chrono::Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let input = i64::deserialize(deserializer)?;

    Ok(chrono::Duration::try_seconds(input).expect("must be fine"))
}

impl Default for TracingBatchExportConfig {
    fn default() -> Self {
        Self {
            scheduled_delay: TracingBatchExportConfig::default_scheduled_delay(),
            max_queue_size: TracingBatchExportConfig::default_max_queue_size(),
            max_export_batch_size: TracingBatchExportConfig::default_max_export_batch_size(),
            max_concurrent_exports: TracingBatchExportConfig::default_max_concurrent_exports(),
        }
    }
}

/// Exporters configuration
#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TracingExportersConfig {
    /// Stdout exporter configuration
    pub stdout: Option<TracingStdoutExporterConfig>,
    /// Otlp exporter configuration
    pub otlp: Option<TracingOtlpExporterConfig>,
    /// Grafbase OTEL exporter configuration when an access token is used.
    #[serde(skip)]
    pub grafbase: Option<TracingOtlpExporterConfig>,
}

/// Stdout exporter configuration
#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TracingStdoutExporterConfig {
    /// Enable or disable the exporter
    #[serde(default)]
    pub enabled: bool,
    /// Batch export configuration
    #[serde(default)]
    pub batch_export: TracingBatchExportConfig,
    /// The maximum duration to export data.
    /// The default value is 60 seconds.
    #[serde(deserialize_with = "deserialize_duration", default = "default_otlp_export_timeout")]
    pub timeout: chrono::Duration,
}

/// Otlp exporter configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TracingOtlpExporterConfig {
    /// Endpoint of the otlp collector
    pub endpoint: Url,
    /// Enable or disable the exporter
    #[serde(default)]
    pub enabled: bool,
    /// Batch export configuration
    #[serde(default)]
    pub batch_export: TracingBatchExportConfig,
    /// Protocol to use when exporting
    #[serde(default)]
    pub protocol: TracingOtlpExporterProtocol,
    /// GRPC exporting configuration
    pub grpc: Option<TracingOtlpExporterGrpcConfig>,
    /// HTTP exporting configuration
    pub http: Option<TracingOtlpExporterHttpConfig>,
    /// The maximum duration to export data.
    /// The default value is 60 seconds.
    #[serde(deserialize_with = "deserialize_duration", default = "default_otlp_export_timeout")]
    pub timeout: chrono::Duration,
}

impl Default for TracingOtlpExporterConfig {
    fn default() -> Self {
        Self {
            endpoint: Url::from_str("http://127.0.0.1:4317").unwrap(),
            enabled: false,
            batch_export: Default::default(),
            protocol: Default::default(),
            grpc: None,
            http: None,
            timeout: default_otlp_export_timeout(),
        }
    }
}

/// OTLP Exporter protocol
#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TracingOtlpExporterProtocol {
    /// GRPC protocol
    #[default]
    Grpc,
    /// HTTP protocol
    Http,
}

/// GRPC exporting configuration
#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TracingOtlpExporterGrpcConfig {
    /// Tls configuration to use on export requests
    pub tls: Option<TracingExporterTlsConfig>,
    /// Headers to send on export requests
    #[serde(default)]
    pub headers: Headers,
}

/// OTLP GRPC TLS export configuration
#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
/// Wraps tls configuration used when exporting data.
/// Any files referenced are read in *sync* fashion using `[std::fs::read]`.
pub struct TracingExporterTlsConfig {
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
impl TryFrom<TracingExporterTlsConfig> for ClientTlsConfig {
    type Error = TracingError;

    fn try_from(value: TracingExporterTlsConfig) -> Result<ClientTlsConfig, Self::Error> {
        use std::fs;

        let mut tls = ClientTlsConfig::new();

        if let Some(domain) = value.domain_name {
            tls = tls.domain_name(domain);
        }

        if let Some(ca) = value.ca {
            let ca_cert = fs::read(ca).map_err(TracingError::FileReadError)?;
            tls = tls.ca_certificate(Certificate::from_pem(ca_cert))
        }

        if let Some(cert) = value.cert {
            let cert = fs::read(cert).map_err(TracingError::FileReadError)?;

            let key = value
                .key
                .map(fs::read)
                .transpose()
                .map_err(TracingError::FileReadError)?
                .unwrap_or_default();

            let identity = Identity::from_pem(cert, key);
            tls = tls.identity(identity);
        }

        Ok(tls)
    }
}

fn default_otlp_export_timeout() -> chrono::Duration {
    chrono::Duration::try_seconds(60).expect("must be fine")
}

/// OTLP HTTP exporting configuration
#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TracingOtlpExporterHttpConfig {
    /// Http headers to send on export requests
    #[serde(default)]
    pub headers: Headers,
}

/// List of headers to be sent on export requests
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Headers(Vec<(HeaderName, HeaderValue)>);
impl Headers {
    /// Consume self and return the inner list
    pub fn into_inner(self) -> Vec<(HeaderName, HeaderValue)> {
        self.0
    }

    /// Gets the headers as a referenced slice
    pub fn inner(&self) -> &[(HeaderName, HeaderValue)] {
        &self.0
    }

    /// Consume self and return a map of header/header_value as ascii strings
    pub fn try_into_map(self) -> Result<HashMap<String, String>, TracingError> {
        self.into_inner()
            .into_iter()
            .map(|(name, value)| {
                let value = value
                    .to_str()
                    .map_err(|err| TracingError::SpanExporterSetup(err.to_string()))?;
                Ok((name.to_string(), value.to_string()))
            })
            .collect::<Result<HashMap<_, _>, _>>()
    }
}

impl From<Vec<(HeaderName, HeaderValue)>> for Headers {
    fn from(headers: Vec<(HeaderName, HeaderValue)>) -> Self {
        Self(headers)
    }
}

impl<'de> Deserialize<'de> for Headers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(HeaderMapVisitor)
    }
}

struct HeaderMapVisitor;
impl<'de> Visitor<'de> for HeaderMapVisitor {
    type Value = Headers;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "a key-value map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut headers = Vec::with_capacity(map.size_hint().unwrap_or(0));

        while let Some((key, value)) = map.next_entry::<String, String>()? {
            let header_name = HeaderName::from_str(&key).map_err(|err| DeserializeError::custom(err.to_string()))?;
            let header_value =
                HeaderValue::from_str(&value).map_err(|err| DeserializeError::custom(err.to_string()))?;

            headers.push((header_name, header_value));
        }

        Ok(Headers(headers))
    }
}

#[cfg(test)]
pub mod tests {
    use std::path::PathBuf;
    use std::str::FromStr;

    use http::{HeaderName, HeaderValue};
    use indoc::indoc;
    use url::Url;

    use tempfile as _;

    use super::{
        Headers, TracingBatchExportConfig, TracingCollectConfig, TracingConfig, TracingExporterTlsConfig,
        TracingOtlpExporterConfig, TracingOtlpExporterGrpcConfig, TracingOtlpExporterHttpConfig,
        TracingOtlpExporterProtocol, TracingStdoutExporterConfig, DEFAULT_FILTER, DEFAULT_SAMPLING,
    };

    #[test]
    fn enabled_defaults() {
        // prepare
        let input = indoc! {r#"
            enabled = true
        "#};

        // act
        let config: TracingConfig = toml::from_str(input).unwrap();

        // assert
        assert_eq!(
            TracingConfig {
                enabled: true,
                sampling: DEFAULT_SAMPLING,
                filter: DEFAULT_FILTER.to_string(),
                ..Default::default()
            },
            config
        );
    }

    #[test]
    fn sampling() {
        // prepare
        let input = indoc! {r#"
            sampling = 0.20
        "#};

        // act
        let config: TracingConfig = toml::from_str(input).unwrap();

        // assert
        assert_eq!(
            TracingConfig {
                sampling: 0.20,
                ..Default::default()
            },
            config
        );
    }

    #[test]
    fn sampling_invalid() {
        // prepare
        let input = indoc! {r#"
            sampling = 1.0121
        "#};

        // act
        let error = toml::from_str::<TracingConfig>(input).unwrap_err();

        // assert
        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 1, column 12
          |
        1 | sampling = 1.0121
          |            ^^^^^^
        input value should be 0..1
        "###);
    }

    #[test]
    fn custom_collect() {
        // prepare
        let input = indoc! {r#"
            [collect]
            max_events_per_span = 1
            max_attributes_per_span = 1
            max_links_per_span = 1
            max_attributes_per_event = 1
            max_attributes_per_link = 1
        "#};

        // act
        let config: TracingConfig = toml::from_str(input).unwrap();

        // assert
        assert_eq!(
            TracingCollectConfig {
                max_events_per_span: 1,
                max_attributes_per_span: 1,
                max_links_per_span: 1,
                max_attributes_per_event: 1,
                max_attributes_per_link: 1,
            },
            config.collect
        );
    }

    #[test]
    fn partial_custom_collect() {
        // prepare
        let input = indoc! {r#"
            [collect]
            max_events_per_span = 1
        "#};

        // act
        let config: TracingConfig = toml::from_str(input).unwrap();

        // assert
        assert_eq!(
            TracingCollectConfig {
                max_events_per_span: 1,
                ..Default::default()
            },
            config.collect
        );
    }

    #[test]
    fn no_exporters() {
        // prepare
        let input = indoc! {r#"
            [exporters]
        "#};

        // act
        let config: TracingConfig = toml::from_str(input).unwrap();

        // assert
        assert!(config.exporters.otlp.is_none());
        assert!(config.exporters.stdout.is_none());
    }

    #[test]
    fn default_otlp_exporter() {
        // prepare
        let input = indoc! {r#"
            [exporters.otlp]
            endpoint = "http://localhost:1234"
        "#};

        // act
        let config: TracingConfig = toml::from_str(input).unwrap();

        // assert
        assert_eq!(
            Some(TracingOtlpExporterConfig {
                endpoint: Url::parse("http://localhost:1234").unwrap(),
                enabled: false,
                batch_export: Default::default(),
                protocol: Default::default(),
                grpc: None,
                http: None,
                timeout: crate::config::default_otlp_export_timeout(),
            }),
            config.exporters.otlp
        );
    }

    #[test]
    fn otlp_exporter_custom_partial_batch_config() {
        // prepare
        let input = indoc! {r#"
            [exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"

            [exporters.otlp.batch_export]
            scheduled_delay = 10
        "#};

        // act
        let config: TracingConfig = toml::from_str(input).unwrap();

        // assert
        assert_eq!(
            Some(TracingOtlpExporterConfig {
                endpoint: Url::parse("http://localhost:1234").unwrap(),
                enabled: true,
                batch_export: TracingBatchExportConfig {
                    scheduled_delay: chrono::Duration::try_seconds(10).expect("must be fine"),
                    ..Default::default()
                },
                ..Default::default()
            }),
            config.exporters.otlp
        );
    }

    #[test]
    fn otlp_exporter_kitchen_sink() {
        // prepare
        let input = indoc! {r#"
            [exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = 120

            [exporters.otlp.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [exporters.otlp.grpc.tls]
            domain_name = "my_domain"
            key = "/certs/grafbase.key"
            ca = "/certs/ca.crt"
            cert = "/certs/grafbase.crt"

            [exporters.otlp.grpc.headers]
            header1 = "header1"

            [exporters.otlp.http.headers]
            header1 = "header1"
        "#};

        // act
        let config: TracingConfig = toml::from_str(input).unwrap();

        // assert
        assert_eq!(
            Some(TracingOtlpExporterConfig {
                endpoint: Url::parse("http://localhost:1234").unwrap(),
                enabled: true,
                batch_export: TracingBatchExportConfig {
                    scheduled_delay: chrono::Duration::try_seconds(10).expect("must be fine"),
                    max_queue_size: 10,
                    max_export_batch_size: 10,
                    max_concurrent_exports: 10,
                },
                protocol: TracingOtlpExporterProtocol::Grpc,
                grpc: Some(TracingOtlpExporterGrpcConfig {
                    tls: Some(TracingExporterTlsConfig {
                        domain_name: Some("my_domain".to_string()),
                        key: Some(PathBuf::from_str("/certs/grafbase.key").unwrap()),
                        ca: Some(PathBuf::from_str("/certs/ca.crt").unwrap()),
                        cert: Some(PathBuf::from_str("/certs/grafbase.crt").unwrap()),
                    }),
                    headers: Headers(vec![(
                        HeaderName::from_str("header1").unwrap(),
                        HeaderValue::from_str("header1").unwrap()
                    )]),
                }),
                http: Some(TracingOtlpExporterHttpConfig {
                    headers: Headers(vec![(
                        HeaderName::from_str("header1").unwrap(),
                        HeaderValue::from_str("header1").unwrap()
                    )]),
                }),
                timeout: chrono::Duration::try_seconds(120).expect("must be fine"),
            }),
            config.exporters.otlp
        );
    }

    #[test]
    fn stdout_exporter_kitchen_sink() {
        // prepare
        let input = indoc! {r#"
            [exporters.stdout]
            enabled = true
            timeout = 10

            [exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10
        "#};

        // act
        let config: TracingConfig = toml::from_str(input).unwrap();

        // assert
        assert_eq!(
            Some(TracingStdoutExporterConfig {
                enabled: true,
                batch_export: TracingBatchExportConfig {
                    scheduled_delay: chrono::Duration::try_seconds(10).expect("must be fine"),
                    max_queue_size: 10,
                    max_export_batch_size: 10,
                    max_concurrent_exports: 10,
                },
                timeout: chrono::Duration::try_seconds(10).expect("must be fine"),
            }),
            config.exporters.stdout
        );
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn tls_config() {
        use crate::error::TracingError;
        use tonic::transport::ClientTlsConfig;

        let tls_config = TracingExporterTlsConfig::default();

        // ok, no error reading file
        let _client_tls_config = ClientTlsConfig::try_from(tls_config).unwrap();

        // error reading ca file
        let tls_config = TracingExporterTlsConfig {
            ca: Some(PathBuf::from_str("/certs/ca.crt").unwrap()),
            ..Default::default()
        };
        let result = ClientTlsConfig::try_from(tls_config);
        assert!(matches!(result.err().unwrap(), TracingError::FileReadError(_)));

        // error reading cert file
        let tls_config = TracingExporterTlsConfig {
            cert: Some(PathBuf::from_str("/certs/grafbase.crt").unwrap()),
            ..Default::default()
        };
        let result = ClientTlsConfig::try_from(tls_config);
        assert!(matches!(result.err().unwrap(), TracingError::FileReadError(_)));

        // error reading key file
        let tmp_cert_file = tempfile::NamedTempFile::new().unwrap();
        let tmp_path = &tmp_cert_file.into_temp_path();
        let tls_config = TracingExporterTlsConfig {
            cert: Some(tmp_path.into()),
            key: Some(PathBuf::from_str("/certs/grafbase.key").unwrap()),
            ..Default::default()
        };
        let result = ClientTlsConfig::try_from(tls_config);
        assert!(matches!(result.err().unwrap(), TracingError::FileReadError(_)));

        // ok, optional key file
        let tmp_cert_file = tempfile::NamedTempFile::new().unwrap();
        let tmp_path = &tmp_cert_file.into_temp_path();
        let tls_config = TracingExporterTlsConfig {
            cert: Some(tmp_path.into()),
            key: None,
            ..Default::default()
        };
        let result = ClientTlsConfig::try_from(tls_config);
        assert!(result.is_ok());

        // ok, full
        let tmp_cert_file = tempfile::NamedTempFile::new().unwrap();
        let tmp_cert_path = tmp_cert_file.into_temp_path();
        let tmp_ca_file = tempfile::NamedTempFile::new().unwrap();
        let tmp_ca_path = tmp_ca_file.into_temp_path();
        let tmp_key_file = tempfile::NamedTempFile::new().unwrap();
        let tmp_key_path = tmp_key_file.into_temp_path();

        let tls_config = TracingExporterTlsConfig {
            ca: Some((&tmp_ca_path).into()),
            cert: Some((&tmp_cert_path).into()),
            key: Some((&tmp_key_path).into()),
            domain_name: Some("domain".to_string()),
        };
        let result = ClientTlsConfig::try_from(tls_config);
        assert!(result.is_ok());
    }
}
