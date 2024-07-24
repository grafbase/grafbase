mod exporters;

use std::collections::HashMap;

// #[cfg(feature = "otlp")]
pub use exporters::{
    Headers, OtlpExporterConfig, OtlpExporterGrpcConfig, OtlpExporterHttpConfig, OtlpExporterProtocol,
    OtlpExporterTlsConfig,
};
pub use exporters::{
    LogsConfig, MetricsConfig, {TracingCollectConfig, TracingConfig, DEFAULT_SAMPLING},
};

pub use exporters::{BatchExportConfig, ExportersConfig, StdoutExporterConfig};

/// Holds telemetry configuration
#[derive(Default, Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryConfig {
    /// The name of the service
    pub service_name: String,
    /// Additional resource attributes
    #[serde(default)]
    pub resource_attributes: HashMap<String, String>,
    /// Global exporters config
    #[serde(default)]
    pub exporters: ExportersConfig,
    /// Separate configuration for logs exports. If set, overrides the global values.
    pub logs: Option<LogsConfig>,
    /// Separate configuration for traces exports. If set, overrides the global values.
    #[serde(default)]
    pub tracing: TracingConfig,
    /// Separate configuration for metrics exports. If set, overrides the global values.
    pub metrics: Option<MetricsConfig>,
    /// Grafbase OTEL exporter configuration when an access token is used.
    #[serde(skip)]
    pub grafbase: Option<OtlpExporterConfig>,
}

impl TelemetryConfig {
    pub fn tracing_stdout_config(&self) -> Option<&StdoutExporterConfig> {
        match self.tracing.exporters.stdout.as_ref() {
            Some(config) if config.enabled => Some(config),
            Some(_) => None,
            None => self.exporters.stdout.as_ref().filter(|c| c.enabled),
        }
    }

    #[cfg(feature = "otlp")]
    pub fn tracing_otlp_config(&self) -> Option<&OtlpExporterConfig> {
        match self.tracing.exporters.otlp.as_ref() {
            Some(config) if config.enabled => Some(config),
            Some(_) => None,
            None => self.exporters.otlp.as_ref().filter(|c| c.enabled),
        }
    }

    pub fn tracing_exporters_enabled(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "otlp")] {
                self.tracing_otlp_config().is_some()
                    || self.tracing_stdout_config().is_some()
                    || self.grafbase_otlp_config().is_some()
            } else {
                self.tracing_stdout_config().is_some()
            }
        }
    }

    pub fn metrics_stdout_config(&self) -> Option<&StdoutExporterConfig> {
        match self.metrics.as_ref().and_then(|c| c.exporters.stdout.as_ref()) {
            Some(config) if config.enabled => Some(config),
            Some(_) => None,
            None => self.exporters.stdout.as_ref().filter(|c| c.enabled),
        }
    }

    #[cfg(feature = "otlp")]
    pub fn metrics_otlp_config(&self) -> Option<&OtlpExporterConfig> {
        match self.metrics.as_ref().and_then(|c| c.exporters.otlp.as_ref()) {
            Some(config) if config.enabled => Some(config),
            Some(_) => None,
            None => self.exporters.otlp.as_ref().filter(|c| c.enabled),
        }
    }

    pub fn logs_stdout_config(&self) -> Option<&StdoutExporterConfig> {
        match self.logs.as_ref().and_then(|c| c.exporters.stdout.as_ref()) {
            Some(config) if config.enabled => Some(config),
            Some(_) => None,
            None => self.exporters.stdout.as_ref().filter(|c| c.enabled),
        }
    }

    #[cfg(feature = "otlp")]
    pub fn logs_otlp_config(&self) -> Option<&OtlpExporterConfig> {
        match self.logs.as_ref().and_then(|c| c.exporters.otlp.as_ref()) {
            Some(config) if config.enabled => Some(config),
            Some(_) => None,
            None => self.exporters.otlp.as_ref().filter(|c| c.enabled),
        }
    }

    pub fn logs_exporters_enabled(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "otlp")] {
                self.logs_otlp_config().is_some() || self.logs_stdout_config().is_some()
            } else {
                self.logs_stdout_config().is_some()
            }
        }
    }

    #[cfg(feature = "otlp")]
    pub fn grafbase_otlp_config(&self) -> Option<&OtlpExporterConfig> {
        self.grafbase.as_ref()
    }
}

#[cfg(test)]
pub mod tests {
    use super::{BatchExportConfig, TracingCollectConfig, TracingConfig};
    use crate::config::StdoutExporterConfig;
    #[cfg(feature = "otlp")]
    use ascii::AsciiString;
    #[cfg(feature = "otlp")]
    use chrono::Duration;
    use indoc::indoc;
    #[cfg(feature = "otlp")]
    use std::path::PathBuf;
    #[cfg(feature = "otlp")]
    use std::str::FromStr;
    use tempfile as _;
    #[cfg(feature = "otlp")]
    use url::Url;

    #[cfg(feature = "otlp")]
    use super::{
        Headers, OtlpExporterConfig, OtlpExporterGrpcConfig, OtlpExporterHttpConfig, OtlpExporterProtocol,
        OtlpExporterTlsConfig,
    };

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

    #[cfg(feature = "otlp")]
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

    #[cfg(feature = "otlp")]
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
            Some(OtlpExporterConfig {
                endpoint: Url::parse("http://localhost:1234").unwrap(),
                enabled: false,
                batch_export: Default::default(),
                protocol: Default::default(),
                grpc: None,
                http: None,
                timeout: Duration::try_seconds(60).unwrap(),
            }),
            config.exporters.otlp
        );
    }

    #[cfg(feature = "otlp")]
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
            Some(OtlpExporterConfig {
                endpoint: Url::parse("http://localhost:1234").unwrap(),
                enabled: true,
                batch_export: BatchExportConfig {
                    scheduled_delay: chrono::Duration::try_seconds(10).expect("must be fine"),
                    ..Default::default()
                },
                ..Default::default()
            }),
            config.exporters.otlp
        );
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn otlp_exporter_kitchen_sink() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

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
        let config: TelemetryConfig = toml::from_str(input).unwrap();

        // assert
        assert_eq!(
            Some(OtlpExporterConfig {
                endpoint: Url::parse("http://localhost:1234").unwrap(),
                enabled: true,
                batch_export: BatchExportConfig {
                    scheduled_delay: chrono::Duration::try_seconds(10).expect("must be fine"),
                    max_queue_size: 10,
                    max_export_batch_size: 10,
                    max_concurrent_exports: 10,
                },
                protocol: OtlpExporterProtocol::Grpc,
                grpc: Some(OtlpExporterGrpcConfig {
                    tls: Some(OtlpExporterTlsConfig {
                        domain_name: Some("my_domain".to_string()),
                        key: Some(PathBuf::from_str("/certs/grafbase.key").unwrap()),
                        ca: Some(PathBuf::from_str("/certs/ca.crt").unwrap()),
                        cert: Some(PathBuf::from_str("/certs/grafbase.crt").unwrap()),
                    }),
                    headers: Headers::from(vec![(
                        AsciiString::from_ascii("header1").unwrap(),
                        AsciiString::from_ascii("header1").unwrap()
                    )]),
                }),
                http: Some(OtlpExporterHttpConfig {
                    headers: Headers::from(vec![(
                        AsciiString::from_ascii("header1").unwrap(),
                        AsciiString::from_ascii("header1").unwrap()
                    )]),
                }),
                timeout: chrono::Duration::try_seconds(120).expect("must be fine"),
            }),
            config.exporters.otlp
        );
    }

    #[test]
    fn tracing_stdout_defaults() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = 10

            [exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.exporters.stdout.as_ref();

        assert_eq!(expected, config.tracing_stdout_config());
        assert!(expected.is_some());
    }

    #[test]
    fn tracing_stdout_alternative_config_not_enabled() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = 10

            [exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [tracing.exporters.stdout]
            enabled = false
            timeout = 9

            [tracing.exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();

        assert_eq!(None, config.tracing_stdout_config());
    }

    #[test]
    fn tracing_stdout_alternative_config_enabled() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = 10

            [exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [tracing.exporters.stdout]
            enabled = true
            timeout = 9

            [tracing.exporters.stdout.batch_export]
            scheduled_delay = 9
            max_queue_size = 9
            max_export_batch_size = 9
            max_concurrent_exports = 9
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.tracing.exporters.stdout.as_ref();

        assert_eq!(expected, config.tracing_stdout_config());
        assert!(expected.is_some());
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn tracing_otlp_default_config() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

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

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.exporters.otlp.as_ref();

        assert_eq!(expected, config.tracing_otlp_config());
        assert!(expected.is_some());
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn tracing_otlp_alternative_config_not_enabled() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

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

            [tracing.exporters.otlp]
            enabled = false
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = 120

            [tracing.exporters.otlp.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [tracing.exporters.otlp.grpc.tls]
            domain_name = "my_domain"
            key = "/certs/grafbase.key"
            ca = "/certs/ca.crt"
            cert = "/certs/grafbase.crt"

            [tracing.exporters.otlp.grpc.headers]
            header1 = "header1"

            [tracing.exporters.otlp.http.headers]
            header1 = "header1"
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();

        assert_eq!(None, config.tracing_otlp_config());
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn tracing_otlp_alternative_config_enabled() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

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

            [tracing.exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = 120

            [tracing.exporters.otlp.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [tracing.exporters.otlp.grpc.tls]
            domain_name = "my_domain"
            key = "/certs/grafbase.key"
            ca = "/certs/ca.crt"
            cert = "/certs/grafbase.crt"

            [tracing.exporters.otlp.grpc.headers]
            header1 = "header1"

            [tracing.exporters.otlp.http.headers]
            header1 = "header1"
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.tracing.exporters.otlp.as_ref();

        assert_eq!(expected, config.tracing_otlp_config());
        assert!(expected.is_some());
    }

    #[test]
    fn metrics_stdout_defaults() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = 10

            [exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.exporters.stdout.as_ref();

        assert_eq!(expected, config.metrics_stdout_config());
        assert!(expected.is_some());
    }

    #[test]
    fn metrics_stdout_alternative_config_not_enabled() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = 10

            [exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [metrics.exporters.stdout]
            enabled = false
            timeout = 9

            [metrics.exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        assert_eq!(None, config.metrics_stdout_config());
    }

    #[test]
    fn metrics_stdout_alternative_config_enabled() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = 10

            [exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [metrics.exporters.stdout]
            enabled = true
            timeout = 9

            [metrics.exporters.stdout.batch_export]
            scheduled_delay = 9
            max_queue_size = 9
            max_export_batch_size = 9
            max_concurrent_exports = 9
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.metrics.as_ref().and_then(|c| c.exporters.stdout.as_ref());

        assert_eq!(expected, config.metrics_stdout_config(),);
        assert!(expected.is_some());
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn metrics_otlp_default_config() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

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

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.exporters.otlp.as_ref();

        assert_eq!(expected, config.metrics_otlp_config());
        assert!(expected.is_some());
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn metrics_otlp_alternative_config_not_enabled() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

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

            [metrics.exporters.otlp]
            enabled = false
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = 120

            [metrics.exporters.otlp.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [metrics.exporters.otlp.grpc.tls]
            domain_name = "my_domain"
            key = "/certs/grafbase.key"
            ca = "/certs/ca.crt"
            cert = "/certs/grafbase.crt"

            [metrics.exporters.otlp.grpc.headers]
            header1 = "header1"

            [metrics.exporters.otlp.http.headers]
            header1 = "header1"
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();

        assert_eq!(None, config.metrics_otlp_config());
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn metrics_otlp_alternative_config_enabled() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

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

            [metrics.exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = 120

            [metrics.exporters.otlp.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [metrics.exporters.otlp.grpc.tls]
            domain_name = "my_domain"
            key = "/certs/grafbase.key"
            ca = "/certs/ca.crt"
            cert = "/certs/grafbase.crt"

            [metrics.exporters.otlp.grpc.headers]
            header1 = "header1"

            [metrics.exporters.otlp.http.headers]
            header1 = "header1"
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.metrics.as_ref().and_then(|c| c.exporters.otlp.as_ref());

        assert_eq!(expected, config.metrics_otlp_config());
        assert!(expected.is_some());
    }

    #[test]
    fn logs_stdout_defaults() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = 10

            [exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.exporters.stdout.as_ref();

        assert_eq!(expected, config.logs_stdout_config());
        assert!(expected.is_some());
    }

    #[test]
    fn logs_stdout_alternative_config_not_enabled() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = 10

            [exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [logs.exporters.stdout]
            enabled = false
            timeout = 9

            [logs.exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();

        assert_eq!(None, config.logs_stdout_config());
    }

    #[test]
    fn logs_stdout_alternative_config_enabled() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = 10

            [exporters.stdout.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [logs.exporters.stdout]
            enabled = true
            timeout = 9

            [logs.exporters.stdout.batch_export]
            scheduled_delay = 9
            max_queue_size = 9
            max_export_batch_size = 9
            max_concurrent_exports = 9
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.logs.as_ref().and_then(|c| c.exporters.stdout.as_ref());

        assert_eq!(expected, config.logs_stdout_config());
        assert!(expected.is_some());
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn logs_otlp_default_config() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

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

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.exporters.otlp.as_ref();

        assert_eq!(expected, config.logs_otlp_config());
        assert!(expected.is_some());
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn logs_otlp_alternative_config_not_enabled() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

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

            [logs.exporters.otlp]
            enabled = false
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = 120

            [logs.exporters.otlp.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [logs.exporters.otlp.grpc.tls]
            domain_name = "my_domain"
            key = "/certs/grafbase.key"
            ca = "/certs/ca.crt"
            cert = "/certs/grafbase.crt"

            [logs.exporters.otlp.grpc.headers]
            header1 = "header1"

            [logs.exporters.otlp.http.headers]
            header1 = "header1"
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        assert_eq!(None, config.logs_otlp_config());
    }

    #[cfg(feature = "otlp")]
    #[test]
    fn logs_otlp_alternative_config_enabled() {
        use crate::config::TelemetryConfig;

        let input = indoc! {r#"
            service_name = "kekw"

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

            [logs.exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = 120

            [logs.exporters.otlp.batch_export]
            scheduled_delay = 10
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [logs.exporters.otlp.grpc.tls]
            domain_name = "my_domain"
            key = "/certs/grafbase.key"
            ca = "/certs/ca.crt"
            cert = "/certs/grafbase.crt"

            [logs.exporters.otlp.grpc.headers]
            header1 = "header1"

            [logs.exporters.otlp.http.headers]
            header1 = "header1"
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.logs.as_ref().and_then(|c| c.exporters.otlp.as_ref());

        assert_eq!(expected, config.logs_otlp_config());
        assert!(expected.is_some());
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
            Some(StdoutExporterConfig {
                enabled: true,
                batch_export: Some(BatchExportConfig {
                    scheduled_delay: chrono::Duration::try_seconds(10).expect("must be fine"),
                    max_queue_size: 10,
                    max_export_batch_size: 10,
                    max_concurrent_exports: 10,
                }),
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

        let tls_config = OtlpExporterTlsConfig::default();

        // ok, no error reading file
        let _client_tls_config = ClientTlsConfig::try_from(tls_config).unwrap();

        // error reading ca file
        let tls_config = OtlpExporterTlsConfig {
            ca: Some(PathBuf::from_str("/certs/ca.crt").unwrap()),
            ..Default::default()
        };
        let result = ClientTlsConfig::try_from(tls_config);
        assert!(matches!(result.err().unwrap(), TracingError::FileReadError(_)));

        // error reading cert file
        let tls_config = OtlpExporterTlsConfig {
            cert: Some(PathBuf::from_str("/certs/grafbase.crt").unwrap()),
            ..Default::default()
        };
        let result = ClientTlsConfig::try_from(tls_config);
        assert!(matches!(result.err().unwrap(), TracingError::FileReadError(_)));

        // error reading key file
        let tmp_cert_file = tempfile::NamedTempFile::new().unwrap();
        let tmp_path = &tmp_cert_file.into_temp_path();
        let tls_config = OtlpExporterTlsConfig {
            cert: Some(tmp_path.into()),
            key: Some(PathBuf::from_str("/certs/grafbase.key").unwrap()),
            ..Default::default()
        };
        let result = ClientTlsConfig::try_from(tls_config);
        assert!(matches!(result.err().unwrap(), TracingError::FileReadError(_)));

        // ok, optional key file
        let tmp_cert_file = tempfile::NamedTempFile::new().unwrap();
        let tmp_path = &tmp_cert_file.into_temp_path();
        let tls_config = OtlpExporterTlsConfig {
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

        let tls_config = OtlpExporterTlsConfig {
            ca: Some((&tmp_ca_path).into()),
            cert: Some((&tmp_cert_path).into()),
            key: Some((&tmp_key_path).into()),
            domain_name: Some("domain".to_string()),
        };
        let result = ClientTlsConfig::try_from(tls_config);
        assert!(result.is_ok());
    }
}
