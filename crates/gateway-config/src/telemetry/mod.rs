pub mod exporters;

use std::collections::HashMap;

use ascii::AsciiString;
pub use exporters::*;

/// Holds telemetry configuration
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TelemetryConfig {
    /// The name of the service
    pub service_name: String,
    /// Additional resource attributes
    pub resource_attributes: HashMap<AsciiString, AsciiString>,
    /// Global exporters config
    pub exporters: GlobalExporterConfig,
    /// Separate configuration for logs exports. If set, overrides the global values.
    pub logs: Option<LogsConfig>,
    /// Separate configuration for traces exports. If set, overrides the global values.
    pub tracing: TracingConfig,
    /// Separate configuration for metrics exports. If set, overrides the global values.
    pub metrics: Option<MetricsConfig>,
    /// Grafbase OTEL exporter configuration when an access token is used.
    #[serde(skip)]
    pub grafbase: Option<OtlpExporterConfig>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        TelemetryConfig {
            service_name: "grafbase-gateway".into(),
            resource_attributes: Default::default(),
            exporters: Default::default(),
            logs: Default::default(),
            tracing: Default::default(),
            metrics: Default::default(),
            grafbase: Default::default(),
        }
    }
}

impl TelemetryConfig {
    pub fn tracing_stdout_config(&self) -> Option<&StdoutExporterConfig> {
        match self.tracing.exporters.stdout.as_ref() {
            Some(config) if config.enabled => Some(config),
            Some(_) => None,
            None => self.exporters.stdout.as_ref().filter(|c| c.enabled),
        }
    }

    pub fn tracing_otlp_config(&self) -> Option<LayeredOtlExporterConfig> {
        let cfg = LayeredOtlExporterConfig {
            global: self.exporters.otlp.clone().unwrap_or_default(),
            local: self.tracing.exporters.otlp.clone().unwrap_or_default(),
        };
        if cfg.is_enabled() { Some(cfg) } else { None }
    }

    pub fn tracing_exporters_enabled(&self) -> bool {
        self.tracing_otlp_config().is_some()
            || self.tracing_stdout_config().is_some()
            || self.grafbase_otlp_config().is_some()
    }

    pub fn metrics_stdout_config(&self) -> Option<&StdoutExporterConfig> {
        match self.metrics.as_ref().and_then(|c| c.exporters.stdout.as_ref()) {
            Some(config) if config.enabled => Some(config),
            Some(_) => None,
            None => self.exporters.stdout.as_ref().filter(|c| c.enabled),
        }
    }

    pub fn metrics_otlp_config(&self) -> Option<LayeredOtlExporterConfig> {
        let cfg = LayeredOtlExporterConfig {
            global: self.exporters.otlp.clone().unwrap_or_default(),
            local: self
                .metrics
                .as_ref()
                .and_then(|cfg| cfg.exporters.otlp.clone())
                .unwrap_or_default(),
        };
        if cfg.is_enabled() { Some(cfg) } else { None }
    }

    pub fn logs_stdout_config(&self) -> Option<&StdoutExporterConfig> {
        match self.logs.as_ref().and_then(|c| c.exporters.stdout.as_ref()) {
            Some(config) if config.enabled => Some(config),
            Some(_) => None,
            None => self.exporters.stdout.as_ref().filter(|c| c.enabled),
        }
    }

    pub fn logs_otlp_config(&self) -> Option<LayeredOtlExporterConfig> {
        let cfg = LayeredOtlExporterConfig {
            global: self.exporters.otlp.clone().unwrap_or_default(),
            local: self
                .logs
                .as_ref()
                .and_then(|cfg| cfg.exporters.otlp.clone())
                .unwrap_or_default(),
        };
        if cfg.is_enabled() { Some(cfg) } else { None }
    }

    pub fn logs_exporters_enabled(&self) -> bool {
        self.logs_otlp_config().is_some() || self.logs_stdout_config().is_some()
    }

    pub fn grafbase_otlp_config(&self) -> Option<OtlpExporterConfig> {
        self.grafbase.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ascii::AsciiString;
    use indoc::indoc;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::time::Duration;
    use url::Url;

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
            Some(OtlpExporterConfig {
                endpoint: Some(Url::parse("http://localhost:1234").unwrap()),
                enabled: None,
                batch_export: Default::default(),
                protocol: Default::default(),
                grpc: None,
                http: None,
                timeout: None
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
            scheduled_delay = "10s"
        "#};

        // act
        let config: TracingConfig = toml::from_str(input).unwrap();

        // assert
        assert_eq!(
            Some(OtlpExporterConfig {
                endpoint: Some(Url::parse("http://localhost:1234").unwrap()),
                enabled: Some(true),
                batch_export: Some(BatchExportConfig {
                    scheduled_delay: Duration::from_secs(10),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            config.exporters.otlp
        );
    }

    #[test]
    fn otlp_exporter_kitchen_sink() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = "120s"

            [exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
                endpoint: Some(Url::parse("http://localhost:1234").unwrap()),
                enabled: Some(true),
                batch_export: Some(BatchExportConfig {
                    scheduled_delay: Duration::from_secs(10),
                    max_queue_size: 10,
                    max_export_batch_size: 10,
                    max_concurrent_exports: 10,
                }),
                protocol: Some(OtlpExporterProtocol::Grpc),
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
                timeout: Some(Duration::from_secs(120)),
            }),
            config.exporters.otlp
        );
    }

    #[test]
    fn tracing_stdout_defaults() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = "10s"

            [exporters.stdout.batch_export]
            scheduled_delay = "10s"
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
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = "10s"

            [exporters.stdout.batch_export]
            scheduled_delay = "10s"
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [tracing.exporters.stdout]
            enabled = false
            timeout = "9s"

            [tracing.exporters.stdout.batch_export]
            scheduled_delay = "10s"
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();

        assert_eq!(None, config.tracing_stdout_config());
    }

    #[test]
    fn tracing_stdout_alternative_config_enabled() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = "10s"

            [exporters.stdout.batch_export]
            scheduled_delay = "10s"
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [tracing.exporters.stdout]
            enabled = true
            timeout = "9s"

            [tracing.exporters.stdout.batch_export]
            scheduled_delay = "9s"
            max_queue_size = 9
            max_export_batch_size = 9
            max_concurrent_exports = 9
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.tracing.exporters.stdout.as_ref();

        assert_eq!(expected, config.tracing_stdout_config());
        assert!(expected.is_some());
    }

    #[test]
    fn tracing_otlp_default_config() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = "120s"

            [exporters.otlp.batch_export]
            scheduled_delay = "10s"
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

        insta::assert_debug_snapshot!(config.tracing_otlp_config(), @r#"
        Some(
            LayeredOtlExporterConfig {
                global: OtlpExporterConfig {
                    enabled: Some(
                        true,
                    ),
                    endpoint: Some(
                        Url {
                            scheme: "http",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: Some(
                                Domain(
                                    "localhost",
                                ),
                            ),
                            port: Some(
                                1234,
                            ),
                            path: "/",
                            query: None,
                            fragment: None,
                        },
                    ),
                    batch_export: Some(
                        BatchExportConfig {
                            scheduled_delay: 10s,
                            max_queue_size: 10,
                            max_export_batch_size: 10,
                            max_concurrent_exports: 10,
                        },
                    ),
                    protocol: Some(
                        Grpc,
                    ),
                    grpc: Some(
                        OtlpExporterGrpcConfig {
                            tls: Some(
                                OtlpExporterTlsConfig {
                                    domain_name: Some(
                                        "my_domain",
                                    ),
                                    key: Some(
                                        "/certs/grafbase.key",
                                    ),
                                    cert: Some(
                                        "/certs/grafbase.crt",
                                    ),
                                    ca: Some(
                                        "/certs/ca.crt",
                                    ),
                                },
                            ),
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    http: Some(
                        OtlpExporterHttpConfig {
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    timeout: Some(
                        120s,
                    ),
                },
                local: OtlpExporterConfig {
                    enabled: None,
                    endpoint: None,
                    batch_export: None,
                    protocol: None,
                    grpc: None,
                    http: None,
                    timeout: None,
                },
            },
        )
        "#);
    }

    #[test]
    fn tracing_otlp_alternative_config_not_enabled() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = "120s"

            [exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
            timeout = "120s"

            [tracing.exporters.otlp.batch_export]
            scheduled_delay = "10s"
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

        insta::assert_debug_snapshot!(config.tracing_otlp_config(), @"None");
    }

    #[test]
    fn tracing_otlp_alternative_config_enabled() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = "120s"

            [exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
            timeout = "120s"

            [tracing.exporters.otlp.batch_export]
            scheduled_delay = "10s"
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

        insta::assert_debug_snapshot!(config.tracing_otlp_config(), @r#"
        Some(
            LayeredOtlExporterConfig {
                global: OtlpExporterConfig {
                    enabled: Some(
                        true,
                    ),
                    endpoint: Some(
                        Url {
                            scheme: "http",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: Some(
                                Domain(
                                    "localhost",
                                ),
                            ),
                            port: Some(
                                1234,
                            ),
                            path: "/",
                            query: None,
                            fragment: None,
                        },
                    ),
                    batch_export: Some(
                        BatchExportConfig {
                            scheduled_delay: 10s,
                            max_queue_size: 10,
                            max_export_batch_size: 10,
                            max_concurrent_exports: 10,
                        },
                    ),
                    protocol: Some(
                        Grpc,
                    ),
                    grpc: Some(
                        OtlpExporterGrpcConfig {
                            tls: Some(
                                OtlpExporterTlsConfig {
                                    domain_name: Some(
                                        "my_domain",
                                    ),
                                    key: Some(
                                        "/certs/grafbase.key",
                                    ),
                                    cert: Some(
                                        "/certs/grafbase.crt",
                                    ),
                                    ca: Some(
                                        "/certs/ca.crt",
                                    ),
                                },
                            ),
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    http: Some(
                        OtlpExporterHttpConfig {
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    timeout: Some(
                        120s,
                    ),
                },
                local: OtlpExporterConfig {
                    enabled: Some(
                        true,
                    ),
                    endpoint: Some(
                        Url {
                            scheme: "http",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: Some(
                                Domain(
                                    "localhost",
                                ),
                            ),
                            port: Some(
                                1234,
                            ),
                            path: "/",
                            query: None,
                            fragment: None,
                        },
                    ),
                    batch_export: Some(
                        BatchExportConfig {
                            scheduled_delay: 10s,
                            max_queue_size: 10,
                            max_export_batch_size: 10,
                            max_concurrent_exports: 10,
                        },
                    ),
                    protocol: Some(
                        Grpc,
                    ),
                    grpc: Some(
                        OtlpExporterGrpcConfig {
                            tls: Some(
                                OtlpExporterTlsConfig {
                                    domain_name: Some(
                                        "my_domain",
                                    ),
                                    key: Some(
                                        "/certs/grafbase.key",
                                    ),
                                    cert: Some(
                                        "/certs/grafbase.crt",
                                    ),
                                    ca: Some(
                                        "/certs/ca.crt",
                                    ),
                                },
                            ),
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    http: Some(
                        OtlpExporterHttpConfig {
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    timeout: Some(
                        120s,
                    ),
                },
            },
        )
        "#);
    }

    #[test]
    fn metrics_stdout_defaults() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = "10s"

            [exporters.stdout.batch_export]
            scheduled_delay = "10s"
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
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = "10s"

            [exporters.stdout.batch_export]
            scheduled_delay = "10s"
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [metrics.exporters.stdout]
            enabled = false
            timeout = "9s"

            [metrics.exporters.stdout.batch_export]
            scheduled_delay = "10s"
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        assert_eq!(None, config.metrics_stdout_config());
    }

    #[test]
    fn metrics_stdout_alternative_config_enabled() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = "10s"

            [exporters.stdout.batch_export]
            scheduled_delay = "10s"
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [metrics.exporters.stdout]
            enabled = true
            timeout = "9s"

            [metrics.exporters.stdout.batch_export]
            scheduled_delay = "9s"
            max_queue_size = 9
            max_export_batch_size = 9
            max_concurrent_exports = 9
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.metrics.as_ref().and_then(|c| c.exporters.stdout.as_ref());

        assert_eq!(expected, config.metrics_stdout_config(),);
        assert!(expected.is_some());
    }

    #[test]
    fn metrics_otlp_default_config() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = "120s"

            [exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
        insta::assert_debug_snapshot!(config.metrics_otlp_config(), @r#"
        Some(
            LayeredOtlExporterConfig {
                global: OtlpExporterConfig {
                    enabled: Some(
                        true,
                    ),
                    endpoint: Some(
                        Url {
                            scheme: "http",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: Some(
                                Domain(
                                    "localhost",
                                ),
                            ),
                            port: Some(
                                1234,
                            ),
                            path: "/",
                            query: None,
                            fragment: None,
                        },
                    ),
                    batch_export: Some(
                        BatchExportConfig {
                            scheduled_delay: 10s,
                            max_queue_size: 10,
                            max_export_batch_size: 10,
                            max_concurrent_exports: 10,
                        },
                    ),
                    protocol: Some(
                        Grpc,
                    ),
                    grpc: Some(
                        OtlpExporterGrpcConfig {
                            tls: Some(
                                OtlpExporterTlsConfig {
                                    domain_name: Some(
                                        "my_domain",
                                    ),
                                    key: Some(
                                        "/certs/grafbase.key",
                                    ),
                                    cert: Some(
                                        "/certs/grafbase.crt",
                                    ),
                                    ca: Some(
                                        "/certs/ca.crt",
                                    ),
                                },
                            ),
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    http: Some(
                        OtlpExporterHttpConfig {
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    timeout: Some(
                        120s,
                    ),
                },
                local: OtlpExporterConfig {
                    enabled: None,
                    endpoint: None,
                    batch_export: None,
                    protocol: None,
                    grpc: None,
                    http: None,
                    timeout: None,
                },
            },
        )
        "#);
    }

    #[test]
    fn metrics_otlp_alternative_config_not_enabled() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = "120s"

            [exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
            timeout = "120s"

            [metrics.exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
        insta::assert_debug_snapshot!(config.metrics_otlp_config(), @"None");
    }

    #[test]
    fn metrics_otlp_alternative_config_enabled() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = "120s"

            [exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
            timeout = "120s"

            [metrics.exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
        insta::assert_debug_snapshot!(config.metrics_otlp_config(), @r#"
        Some(
            LayeredOtlExporterConfig {
                global: OtlpExporterConfig {
                    enabled: Some(
                        true,
                    ),
                    endpoint: Some(
                        Url {
                            scheme: "http",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: Some(
                                Domain(
                                    "localhost",
                                ),
                            ),
                            port: Some(
                                1234,
                            ),
                            path: "/",
                            query: None,
                            fragment: None,
                        },
                    ),
                    batch_export: Some(
                        BatchExportConfig {
                            scheduled_delay: 10s,
                            max_queue_size: 10,
                            max_export_batch_size: 10,
                            max_concurrent_exports: 10,
                        },
                    ),
                    protocol: Some(
                        Grpc,
                    ),
                    grpc: Some(
                        OtlpExporterGrpcConfig {
                            tls: Some(
                                OtlpExporterTlsConfig {
                                    domain_name: Some(
                                        "my_domain",
                                    ),
                                    key: Some(
                                        "/certs/grafbase.key",
                                    ),
                                    cert: Some(
                                        "/certs/grafbase.crt",
                                    ),
                                    ca: Some(
                                        "/certs/ca.crt",
                                    ),
                                },
                            ),
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    http: Some(
                        OtlpExporterHttpConfig {
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    timeout: Some(
                        120s,
                    ),
                },
                local: OtlpExporterConfig {
                    enabled: Some(
                        true,
                    ),
                    endpoint: Some(
                        Url {
                            scheme: "http",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: Some(
                                Domain(
                                    "localhost",
                                ),
                            ),
                            port: Some(
                                1234,
                            ),
                            path: "/",
                            query: None,
                            fragment: None,
                        },
                    ),
                    batch_export: Some(
                        BatchExportConfig {
                            scheduled_delay: 10s,
                            max_queue_size: 10,
                            max_export_batch_size: 10,
                            max_concurrent_exports: 10,
                        },
                    ),
                    protocol: Some(
                        Grpc,
                    ),
                    grpc: Some(
                        OtlpExporterGrpcConfig {
                            tls: Some(
                                OtlpExporterTlsConfig {
                                    domain_name: Some(
                                        "my_domain",
                                    ),
                                    key: Some(
                                        "/certs/grafbase.key",
                                    ),
                                    cert: Some(
                                        "/certs/grafbase.crt",
                                    ),
                                    ca: Some(
                                        "/certs/ca.crt",
                                    ),
                                },
                            ),
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    http: Some(
                        OtlpExporterHttpConfig {
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    timeout: Some(
                        120s,
                    ),
                },
            },
        )
        "#);
    }

    #[test]
    fn logs_stdout_defaults() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = "10s"

            [exporters.stdout.batch_export]
            scheduled_delay = "10s"
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
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = "10s"

            [exporters.stdout.batch_export]
            scheduled_delay = "10s"
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [logs.exporters.stdout]
            enabled = false
            timeout = "9s"

            [logs.exporters.stdout.batch_export]
            scheduled_delay = "10s"
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();

        assert_eq!(None, config.logs_stdout_config());
    }

    #[test]
    fn logs_stdout_alternative_config_enabled() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.stdout]
            enabled = true
            timeout = "10s"

            [exporters.stdout.batch_export]
            scheduled_delay = "10s"
            max_queue_size = 10
            max_export_batch_size = 10
            max_concurrent_exports = 10

            [logs.exporters.stdout]
            enabled = true
            timeout = "9s"

            [logs.exporters.stdout.batch_export]
            scheduled_delay = "9s"
            max_queue_size = 9
            max_export_batch_size = 9
            max_concurrent_exports = 9
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();
        let expected = config.logs.as_ref().and_then(|c| c.exporters.stdout.as_ref());

        assert_eq!(expected, config.logs_stdout_config());
        assert!(expected.is_some());
    }

    #[test]
    fn logs_otlp_default_config() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = "120s"

            [exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
        insta::assert_debug_snapshot!(config.logs_otlp_config(), @r#"
        Some(
            LayeredOtlExporterConfig {
                global: OtlpExporterConfig {
                    enabled: Some(
                        true,
                    ),
                    endpoint: Some(
                        Url {
                            scheme: "http",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: Some(
                                Domain(
                                    "localhost",
                                ),
                            ),
                            port: Some(
                                1234,
                            ),
                            path: "/",
                            query: None,
                            fragment: None,
                        },
                    ),
                    batch_export: Some(
                        BatchExportConfig {
                            scheduled_delay: 10s,
                            max_queue_size: 10,
                            max_export_batch_size: 10,
                            max_concurrent_exports: 10,
                        },
                    ),
                    protocol: Some(
                        Grpc,
                    ),
                    grpc: Some(
                        OtlpExporterGrpcConfig {
                            tls: Some(
                                OtlpExporterTlsConfig {
                                    domain_name: Some(
                                        "my_domain",
                                    ),
                                    key: Some(
                                        "/certs/grafbase.key",
                                    ),
                                    cert: Some(
                                        "/certs/grafbase.crt",
                                    ),
                                    ca: Some(
                                        "/certs/ca.crt",
                                    ),
                                },
                            ),
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    http: Some(
                        OtlpExporterHttpConfig {
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    timeout: Some(
                        120s,
                    ),
                },
                local: OtlpExporterConfig {
                    enabled: None,
                    endpoint: None,
                    batch_export: None,
                    protocol: None,
                    grpc: None,
                    http: None,
                    timeout: None,
                },
            },
        )
        "#);
    }

    #[test]
    fn logs_otlp_alternative_config_not_enabled() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = "120s"

            [exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
            timeout = "120s"

            [logs.exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
        insta::assert_debug_snapshot!(config.logs_otlp_config(), @"None");
    }

    #[test]
    fn logs_otlp_alternative_config_enabled() {
        let input = indoc! {r#"
            service_name = "kekw"

            [exporters.otlp]
            enabled = true
            endpoint = "http://localhost:1234"
            protocol = "grpc"
            timeout = "120s"

            [exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
            timeout = "120s"

            [logs.exporters.otlp.batch_export]
            scheduled_delay = "10s"
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
        insta::assert_debug_snapshot!(config.logs_otlp_config(), @r#"
        Some(
            LayeredOtlExporterConfig {
                global: OtlpExporterConfig {
                    enabled: Some(
                        true,
                    ),
                    endpoint: Some(
                        Url {
                            scheme: "http",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: Some(
                                Domain(
                                    "localhost",
                                ),
                            ),
                            port: Some(
                                1234,
                            ),
                            path: "/",
                            query: None,
                            fragment: None,
                        },
                    ),
                    batch_export: Some(
                        BatchExportConfig {
                            scheduled_delay: 10s,
                            max_queue_size: 10,
                            max_export_batch_size: 10,
                            max_concurrent_exports: 10,
                        },
                    ),
                    protocol: Some(
                        Grpc,
                    ),
                    grpc: Some(
                        OtlpExporterGrpcConfig {
                            tls: Some(
                                OtlpExporterTlsConfig {
                                    domain_name: Some(
                                        "my_domain",
                                    ),
                                    key: Some(
                                        "/certs/grafbase.key",
                                    ),
                                    cert: Some(
                                        "/certs/grafbase.crt",
                                    ),
                                    ca: Some(
                                        "/certs/ca.crt",
                                    ),
                                },
                            ),
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    http: Some(
                        OtlpExporterHttpConfig {
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    timeout: Some(
                        120s,
                    ),
                },
                local: OtlpExporterConfig {
                    enabled: Some(
                        true,
                    ),
                    endpoint: Some(
                        Url {
                            scheme: "http",
                            cannot_be_a_base: false,
                            username: "",
                            password: None,
                            host: Some(
                                Domain(
                                    "localhost",
                                ),
                            ),
                            port: Some(
                                1234,
                            ),
                            path: "/",
                            query: None,
                            fragment: None,
                        },
                    ),
                    batch_export: Some(
                        BatchExportConfig {
                            scheduled_delay: 10s,
                            max_queue_size: 10,
                            max_export_batch_size: 10,
                            max_concurrent_exports: 10,
                        },
                    ),
                    protocol: Some(
                        Grpc,
                    ),
                    grpc: Some(
                        OtlpExporterGrpcConfig {
                            tls: Some(
                                OtlpExporterTlsConfig {
                                    domain_name: Some(
                                        "my_domain",
                                    ),
                                    key: Some(
                                        "/certs/grafbase.key",
                                    ),
                                    cert: Some(
                                        "/certs/grafbase.crt",
                                    ),
                                    ca: Some(
                                        "/certs/ca.crt",
                                    ),
                                },
                            ),
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    http: Some(
                        OtlpExporterHttpConfig {
                            headers: Headers(
                                [
                                    (
                                        "header1",
                                        "header1",
                                    ),
                                ],
                            ),
                        },
                    ),
                    timeout: Some(
                        120s,
                    ),
                },
            },
        )
        "#);
    }

    #[test]
    fn stdout_exporter_kitchen_sink() {
        // prepare
        let input = indoc! {r#"
            [exporters.stdout]
            enabled = true
            timeout = "10s"

            [exporters.stdout.batch_export]
            scheduled_delay = "10s"
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
                    scheduled_delay: Duration::from_secs(10),
                    max_queue_size: 10,
                    max_export_batch_size: 10,
                    max_concurrent_exports: 10,
                }),
                timeout: Duration::from_secs(10),
            }),
            config.exporters.stdout
        );
    }

    #[test]
    fn tls_config() {
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
        assert!(result.is_err());

        // error reading cert file
        let tls_config = OtlpExporterTlsConfig {
            cert: Some(PathBuf::from_str("/certs/grafbase.crt").unwrap()),
            ..Default::default()
        };
        let result = ClientTlsConfig::try_from(tls_config);
        assert!(result.is_err());

        // error reading key file
        let tmp_cert_file = tempfile::NamedTempFile::new().unwrap();
        let tmp_path = &tmp_cert_file.into_temp_path();
        let tls_config = OtlpExporterTlsConfig {
            cert: Some(tmp_path.into()),
            key: Some(PathBuf::from_str("/certs/grafbase.key").unwrap()),
            ..Default::default()
        };
        let result = ClientTlsConfig::try_from(tls_config);
        assert!(result.is_err());

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

    #[test]
    fn tracing_with_parent_based_sampling_and_propagation() {
        let input = indoc! {r#"
            [tracing]
            parent_based_sampler = true

            [tracing.propagation]
            trace_context = true
            baggage = true
        "#};

        let config: TelemetryConfig = toml::from_str(input).unwrap();

        assert_eq!(
            TelemetryConfig {
                service_name: "grafbase-gateway".into(),
                resource_attributes: Default::default(),
                exporters: Default::default(),
                logs: Default::default(),
                metrics: Default::default(),
                grafbase: None,
                tracing: TracingConfig {
                    sampling: 0.15,
                    parent_based_sampler: true,
                    collect: Default::default(),
                    exporters: Default::default(),
                    propagation: exporters::PropagationConfig {
                        trace_context: true,
                        baggage: true,
                        aws_xray: false
                    },
                },
            },
            config
        );
    }
}
