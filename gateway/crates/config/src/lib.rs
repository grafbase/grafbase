use grafbase_workspace_hack as _;

pub mod authentication;
pub mod cors;
pub mod entity_caching;
pub mod header;
pub mod health;
pub mod hooks;
pub mod rate_limit;
mod size_ext;
pub mod telemetry;

use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf, time::Duration};

use ascii::AsciiString;
pub use authentication::*;
pub use cors::*;
pub use entity_caching::*;
pub use header::*;
pub use health::*;
pub use hooks::*;
pub use rate_limit::*;
use serde_dynamic_string::DynamicString;
use size::Size;
pub use telemetry::*;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
/// Configuration struct to define settings for self-hosted
/// Grafbase gateway.
pub struct Config {
    /// Graph location and features, such as introspection
    pub graph: GraphConfig,
    /// Server bind settings
    pub network: NetworkConfig,
    /// General settings for the gateway server
    pub gateway: GatewayConfig,
    /// Maximum size of the request body in bytes
    #[serde(deserialize_with = "size_ext::deserialize_positive_size")]
    pub request_body_limit: Size,
    /// Cross-site request forgery settings
    pub csrf: CsrfConfig,
    /// Cross-origin resource sharing settings
    pub cors: Option<CorsConfig>,
    /// Server TLS settings
    pub tls: Option<TlsConfig>,
    /// Graph operation limit settings
    pub operation_limits: Option<OperationLimitsConfig>,
    /// Telemetry settings
    pub telemetry: TelemetryConfig,
    /// Configuration for Trusted Documents.
    pub trusted_documents: TrustedDocumentsConfig,
    /// Authentication configuration
    pub authentication: Option<AuthenticationConfig>,
    /// Header bypass configuration
    pub headers: Vec<HeaderRule>,
    /// Subgraph configuration
    pub subgraphs: BTreeMap<String, SubgraphConfig>,
    /// Hooks configuration
    pub hooks: Option<HooksWasiConfig>,
    /// Health check endpoint configuration
    pub health: HealthConfig,
    /// Global configuration for entity caching
    pub entity_caching: EntityCachingConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            graph: Default::default(),
            network: Default::default(),
            gateway: Default::default(),
            request_body_limit: Size::from_mebibytes(2),
            csrf: Default::default(),
            cors: Default::default(),
            tls: Default::default(),
            operation_limits: Default::default(),
            telemetry: Default::default(),
            trusted_documents: Default::default(),
            authentication: Default::default(),
            headers: Default::default(),
            subgraphs: Default::default(),
            hooks: Default::default(),
            health: Default::default(),
            entity_caching: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BatchingConfig {
    /// If batching should be enabled.
    pub enabled: bool,
    /// How many queries can a batch have.
    pub limit: Option<u8>,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GatewayConfig {
    /// Timeout for gateway requests.
    #[serde(deserialize_with = "duration_str::deserialize_option_duration")]
    pub timeout: Option<Duration>,
    /// Default timeout for subgraph requests.
    #[serde(deserialize_with = "duration_str::deserialize_option_duration")]
    pub subgraph_timeout: Option<Duration>,
    /// Global rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
    /// Global retry configuration
    pub retry: RetryConfig,
    /// Access logs configuration
    pub access_logs: AccessLogsConfig,
    /// Query batching configuration
    pub batching: BatchingConfig,
}

#[derive(Debug, Default, serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RotateMode {
    /// Never rotate
    #[default]
    Never,
    /// A new file every minute
    Minutely,
    /// A new file every hour
    Hourly,
    /// A new file every day
    Daily,
    /// A new size when the current file has reached a certain size
    #[serde(deserialize_with = "size_ext::deserialize_positive_size")]
    Size(Size),
}

#[derive(Debug, Default, serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum LogMode {
    /// The log channel will block if it's full.
    #[default]
    Blocking,
    /// The log channel will return the data back to the caller, if full.
    NonBlocking,
}

#[derive(Debug, Default, serde::Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct AccessLogsConfig {
    /// Enable writing access logs
    pub enabled: bool,
    /// The path to the access log files
    pub path: PathBuf,
    /// How often logs are rotated.
    pub rotate: RotateMode,
    /// What happens if the log channel is full
    pub mode: LogMode,
}

impl AccessLogsConfig {
    pub fn lossy_log(&self) -> bool {
        matches!(self.mode, LogMode::NonBlocking)
    }
}

#[derive(Debug, Default, serde::Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct SubgraphConfig {
    /// URL of the subgraph, overriding the one specified in the federated SDL.
    pub url: Option<Url>,
    /// Header bypass configuration
    pub headers: Vec<HeaderRule>,
    /// The URL to use for GraphQL websocket calls.
    pub websocket_url: Option<Url>,
    /// Rate limiting configuration specifically for this Subgraph
    pub rate_limit: Option<GraphRateLimit>,
    /// Timeout for subgraph requests in seconds. Default: 30 seconds.
    #[serde(deserialize_with = "duration_str::deserialize_option_duration")]
    pub timeout: Option<Duration>,
    pub retry: Option<RetryConfig>,
    /// Subgraph specific entity caching config  this overrides the global config if there
    /// is any
    pub entity_caching: Option<EntityCachingConfig>,
}

#[derive(Debug, serde::Deserialize, Clone, Copy, Default, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct RetryConfig {
    /// Should we retry or not.
    pub enabled: bool,
    /// How many retries are available per second, at a minimum.
    pub min_per_second: Option<u32>,
    /// Each successful request to the subgraph adds to the retry budget. This setting controls for how long the budget remembers successful requests.
    #[serde(deserialize_with = "duration_str::deserialize_option_duration")]
    pub ttl: Option<Duration>,
    /// The fraction of the successful requests budget that can be used for retries.
    pub retry_percent: Option<f32>,
    /// Whether mutations should be retried at all. False by default.
    pub retry_mutations: bool,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GraphConfig {
    pub path: Option<String>,
    pub introspection: bool,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CsrfConfig {
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkConfig {
    pub listen_address: Option<SocketAddr>,
}

#[derive(Debug, serde::Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TlsConfig {
    pub certificate: PathBuf,
    pub key: PathBuf,
}

#[derive(Debug, serde::Deserialize, Default, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct TrustedDocumentsConfig {
    /// If true, the engine will only accept trusted document queries. Default: false.
    pub enabled: bool,
    /// See [BypassHeader]
    #[serde(flatten)]
    pub bypass_header: BypassHeader,
}

/// An optional header that can be passed by clients to bypass trusted documents enforcement, allowing arbitrary queries.
#[derive(Debug, serde::Deserialize, Clone, Default)]
#[serde(default, deny_unknown_fields)]
pub struct BypassHeader {
    /// Name of the optional header that can be set to bypass trusted documents enforcement, when `enabled = true`. Only meaningful in combination with `bypass_header_value`.
    pub bypass_header_name: Option<AsciiString>,
    /// Value of the optional header that can be set to bypass trusted documents enforcement, when `enabled = true`. Only meaningful in combination with `bypass_header_value`.
    pub bypass_header_value: Option<DynamicString<String>>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OperationLimitsConfig {
    /// Limits the deepest nesting of selection sets in an operation,
    /// including fields in fragments.
    pub depth: Option<u16>,
    /// Limits the number of unique fields included in an operation,
    /// including fields of fragments. If a particular field is included
    /// multiple times via aliases, it's counted only once.
    pub height: Option<u16>,
    /// Limits the total number of aliased fields in an operation,
    /// including fields of fragments.
    pub aliases: Option<u16>,
    /// Limits the number of root fields in an operation, including root
    /// fields in fragments. If a particular root field is included multiple
    /// times via aliases, each usage is counted.
    pub root_fields: Option<u16>,
    /// Query complexity takes the number of fields as well as the depth and
    /// any pagination arguments into account. Every scalar field adds 1 point,
    /// every nested field adds 2 points, and every pagination argument multiplies
    /// the nested objects score by the number of records fetched.
    pub complexity: Option<u16>,
}

#[cfg(test)]
mod tests {
    use super::*;

    use ascii::AsciiString;
    use indoc::indoc;
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
    use std::time::Duration;

    #[test]
    fn request_body_limit() {
        let config: Config = toml::from_str(indoc! {r#"
            request_body_limit = "2mB"
        "#})
        .unwrap();
        assert_eq!(Size::from_megabytes(2), config.request_body_limit);

        let config: Config = toml::from_str(indoc! {r#"
            request_body_limit = 2_000_000
        "#})
        .unwrap();
        assert_eq!(Size::from_megabytes(2), config.request_body_limit);

        let config: Config = toml::from_str(indoc! {r#"
            request_body_limit = "2000kB"
        "#})
        .unwrap();
        assert_eq!(Size::from_megabytes(2), config.request_body_limit);

        let config: Config = toml::from_str(indoc! {r#"
            request_body_limit = "2MiB"
        "#})
        .unwrap();
        assert_eq!(Size::from_mebibytes(2), config.request_body_limit);

        let error = toml::from_str::<Config>(indoc! {r#"
            request_body_limit = -123
        "#})
        .unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 1, column 22
          |
        1 | request_body_limit = -123
          |                      ^^^^
        size must be positive
        "###);
    }

    #[test]
    fn network_ipv4() {
        let input = indoc! {r#"
            [network]
            listen_address = "0.0.0.0:4000"
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let expected = Some(SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 4000));

        assert_eq!(expected, config.network.listen_address);
    }

    #[test]
    fn network_ipv6() {
        let input = indoc! {r#"
            [network]
            listen_address = "[::1]:4000"
        "#};

        let config: Config = toml::from_str(input).unwrap();

        let expected = Some(SocketAddr::new(
            std::net::IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)),
            4000,
        ));

        assert_eq!(expected, config.network.listen_address);
    }

    #[test]
    fn graph_defaults() {
        let config: Config = toml::from_str("").unwrap();

        assert!(!config.graph.introspection);
        assert_eq!(None, config.graph.path.as_deref());
    }

    #[test]
    fn graph_values() {
        let input = indoc! {r#"
            [graph]
            path = "/enterprise"
            introspection = true
        "#};

        let config: Config = toml::from_str(input).unwrap();

        assert!(config.graph.introspection);
        assert_eq!(Some("/enterprise"), config.graph.path.as_deref());
    }

    #[test]
    fn csrf_defaults() {
        let config: Config = toml::from_str("").unwrap();

        assert!(!config.csrf.enabled);
    }

    #[test]
    fn csrf() {
        let input = indoc! {r#"
            [csrf]
            enabled = true
        "#};

        let config: Config = toml::from_str(input).unwrap();

        assert!(config.csrf.enabled);
    }

    #[test]
    fn cors_allow_credentials() {
        let input = indoc! {r#"
            [cors]
            allow_credentials = true
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert!(cors.allow_credentials);
    }

    #[test]
    fn cors_allow_credentials_default() {
        let input = indoc! {r#"
            [cors]
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert!(!cors.allow_credentials);
    }

    #[test]
    fn cors_max_age() {
        let input = indoc! {r#"
           [cors]
           max_age = "60s"
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert_eq!(Some(Duration::from_secs(60)), cors.max_age);
    }

    #[test]
    fn cors_allow_origins_default() {
        let input = indoc! {r#"
            [cors]
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert_eq!(None, cors.allow_origins)
    }

    #[test]
    fn cors_allow_origins_any() {
        let input = indoc! {r#"
            [cors]
            allow_origins = "any"
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert_eq!(Some(AnyOrUrlArray::Any), cors.allow_origins)
    }

    #[test]
    fn cors_allow_origins_explicit() {
        let input = indoc! {r#"
            [cors]
            allow_origins = ["https://app.grafbase.com"]
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();
        let expected = AnyOrUrlArray::Explicit(vec!["https://app.grafbase.com".parse().unwrap()]);

        assert_eq!(Some(expected), cors.allow_origins)
    }

    #[test]
    fn cors_allow_origins_invalid_url() {
        let input = indoc! {r#"
            [cors]
            allow_origins = ["foo"]
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 2, column 17
          |
        2 | allow_origins = ["foo"]
          |                 ^^^^^^^
        expecting string "any", or an array of urls
        "###);
    }

    #[test]
    fn cors_allow_methods_default() {
        let input = indoc! {r#"
            [cors]
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert_eq!(None, cors.allow_methods)
    }

    #[test]
    fn cors_allow_methods_any() {
        let input = indoc! {r#"
            [cors]
            allow_methods = "any"
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert_eq!(Some(AnyOrHttpMethodArray::Any), cors.allow_methods)
    }

    #[test]
    fn cors_allow_methods_explicit() {
        let input = indoc! {r#"
            [cors]
            allow_methods = ["POST"]
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();
        let expected = AnyOrHttpMethodArray::Explicit(vec![HttpMethod::Post]);

        assert_eq!(Some(expected), cors.allow_methods)
    }

    #[test]
    fn cors_allow_methods_invalid_method() {
        let input = indoc! {r#"
            [cors]
            allow_methods = ["MEOW"]
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 2, column 17
          |
        2 | allow_methods = ["MEOW"]
          |                 ^^^^^^^^
        expecting string "any", or an array of capitalized HTTP methods
        "###);
    }

    #[test]
    fn cors_allow_headers_default() {
        let input = indoc! {r#"
            [cors]
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert_eq!(None, cors.allow_headers)
    }

    #[test]
    fn cors_allow_headers_any() {
        let input = indoc! {r#"
            [cors]
            allow_headers = "any"
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert_eq!(Some(AnyOrAsciiStringArray::Any), cors.allow_headers)
    }

    #[test]
    fn cors_allow_headers_explicit() {
        let input = indoc! {r#"
            [cors]
            allow_headers = ["Content-Type"]
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        let expected = AnyOrAsciiStringArray::Explicit(vec![AsciiString::from_ascii(b"Content-Type").unwrap()]);

        assert_eq!(Some(expected), cors.allow_headers)
    }

    #[test]
    fn cors_allow_headers_invalid() {
        let input = indoc! {r#"
            [cors]
            allow_headers = ["😂😂😂"]
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 2, column 17
          |
        2 | allow_headers = ["😂😂😂"]
          |                 ^^^^^^^^^^^^^^^^
        expecting string "any", or an array of ASCII strings
        "###);
    }

    #[test]
    fn cors_expose_headers_default() {
        let input = indoc! {r#"
            [cors]
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert_eq!(None, cors.expose_headers);
    }

    #[test]
    fn cors_expose_headers_any() {
        let input = indoc! {r#"
            [cors]
            expose_headers = "any"
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert_eq!(Some(AnyOrAsciiStringArray::Any), cors.expose_headers);
    }

    #[test]
    fn cors_expose_headers_explicit() {
        let input = indoc! {r#"
            [cors]
            expose_headers = ["Content-Type"]
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        let expected = AnyOrAsciiStringArray::Explicit(vec![AsciiString::from_ascii(b"Content-Type").unwrap()]);

        assert_eq!(Some(expected), cors.expose_headers);
    }

    #[test]
    fn cors_expose_headers_invalid() {
        let input = indoc! {r#"
            [cors]
            expose_headers = ["😂😂😂"]
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 2, column 18
          |
        2 | expose_headers = ["😂😂😂"]
          |                  ^^^^^^^^^^^^^^^^
        expecting string "any", or an array of ASCII strings
        "###);
    }

    #[test]
    fn cors_allow_private_network_default() {
        let input = indoc! {r#"
            [cors]
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert!(!cors.allow_private_network);
    }

    #[test]
    fn cors_allow_private_network_explicit() {
        let input = indoc! {r#"
            [cors]
            allow_private_network = true
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let cors = config.cors.unwrap();

        assert!(cors.allow_private_network);
    }

    #[test]
    fn operation_limits() {
        let input = indoc! {r#"
            [operation_limits]
            depth = 3
            height = 10
            aliases = 100
            root_fields = 10
            complexity = 1000
        "#};

        let config: Config = toml::from_str(input).unwrap();
        let operation_limits = config.operation_limits.unwrap();

        let expected = OperationLimitsConfig {
            depth: Some(3),
            height: Some(10),
            aliases: Some(100),
            root_fields: Some(10),
            complexity: Some(1000),
        };

        assert_eq!(expected, operation_limits);
    }

    #[test]
    fn operation_limits_with_too_big_values() {
        let input = indoc! {r#"
            [operation_limits]
            depth = 3
            height = 10
            aliases = 1000000000000000000
            root_fields = 10
            complexity = 1000
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 4, column 11
          |
        4 | aliases = 1000000000000000000
          |           ^^^^^^^^^^^^^^^^^^^
        invalid value: integer `1000000000000000000`, expected u16
        "###);
    }

    #[test]
    fn trusted_documents_omitted() {
        let input = "";

        let config = toml::from_str::<Config>(input).unwrap();

        insta::assert_debug_snapshot!(config.trusted_documents, @r###"
        TrustedDocumentsConfig {
            enabled: false,
            bypass_header: BypassHeader {
                bypass_header_name: None,
                bypass_header_value: None,
            },
        }
        "###)
    }

    #[test]
    fn trusted_documents_just_enabled() {
        let input = indoc! {r#"
            [trusted_documents]
            enabled = true
        "#};

        let config = toml::from_str::<Config>(input).unwrap();

        insta::assert_debug_snapshot!(config.trusted_documents, @r###"
        TrustedDocumentsConfig {
            enabled: true,
            bypass_header: BypassHeader {
                bypass_header_name: None,
                bypass_header_value: None,
            },
        }
        "###)
    }

    #[test]
    fn trusted_documents_bypass_header_value_from_env_var() {
        let input = r###"
            [trusted_documents]
            enabled = true
            bypass_header_name = "my-header-name"
            bypass_header_value = "secret-{{ env.TEST_HEADER_SECRET }}"
        "###;

        let err = toml::from_str::<Config>(input).unwrap_err().to_string();

        insta::assert_snapshot!(err, @r###"
        TOML parse error at line 2, column 13
          |
        2 |             [trusted_documents]
          |             ^^^^^^^^^^^^^^^^^^^
        environment variable not found: `TEST_HEADER_SECRET`
        "###);
    }

    #[test]
    fn trusted_documents_all_settings() {
        let input = r###"
            [trusted_documents]
            enabled = true # default: false
            bypass_header_name = "my-header-name" # default null
            bypass_header_value = "my-secret-value" # default null
        "###;

        let config = toml::from_str::<Config>(input).unwrap();

        insta::assert_debug_snapshot!(config.trusted_documents, @r###"
        TrustedDocumentsConfig {
            enabled: true,
            bypass_header: BypassHeader {
                bypass_header_name: Some(
                    "my-header-name",
                ),
                bypass_header_value: Some(
                    DynamicString(
                        "my-secret-value",
                    ),
                ),
            },
        }
        "###);
    }

    #[test]
    fn trusted_documents_unknown_setting() {
        let input = indoc! {r#"
            [trusted_documents]
            copacetic = false
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();
        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 1, column 1
          |
        1 | [trusted_documents]
          | ^^^^^^^^^^^^^^^^^^^
        unknown field `copacetic`
        "###);
    }

    #[test]
    fn authentication_config() {
        let input = indoc! {r#"
            [[authentication.providers]]

            [authentication.providers.jwt]
            name = "foo"

            [authentication.providers.jwt.jwks]
            url = "https://example.com/.well-known/jwks.json"
            issuer = "https://example.com/"
            audience = "my-project"
            poll_interval = "60s"
        "#};

        let result: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&result.authentication.unwrap(), @r###"
        AuthenticationConfig {
            providers: [
                Jwt(
                    JwtProvider {
                        name: Some(
                            "foo",
                        ),
                        jwks: JwksConfig {
                            url: Url {
                                scheme: "https",
                                cannot_be_a_base: false,
                                username: "",
                                password: None,
                                host: Some(
                                    Domain(
                                        "example.com",
                                    ),
                                ),
                                port: None,
                                path: "/.well-known/jwks.json",
                                query: None,
                                fragment: None,
                            },
                            issuer: Some(
                                "https://example.com/",
                            ),
                            audience: Some(
                                "my-project",
                            ),
                            poll_interval: 60s,
                        },
                        header: AuthenticationHeader {
                            name: "Authorization",
                            value_prefix: "Bearer ",
                        },
                    },
                ),
            ],
        }
        "###);
    }

    #[test]
    fn authentication_invalid_header_name() {
        let input = indoc! {r#"
            [[authentication.providers]]

            [authentication.providers.jwt]
            name = "foo"

            [authentication.providers.jwt.jwks]
            url = "https://example.com/.well-known/jwks.json"
            issuer = "https://example.com/"
            audience = "my-project"
            poll_interval = "60s"

            [authentication.providers.jwt.header]
            name = "Authoriz🎠"
            value_prefix = "Bearer "
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 13, column 8
           |
        13 | name = "Authoriz🎠"
           |        ^^^^^^^^^^^^^^
        invalid value: string "Authoriz🎠", expected an ascii string
        "###);
    }

    #[test]
    fn authentication_invalid_header_value() {
        let input = indoc! {r#"
            [[authentication.providers]]

            [authentication.providers.jwt]
            name = "foo"

            [authentication.providers.jwt.jwks]
            url = "https://example.com/.well-known/jwks.json"
            issuer = "https://example.com/"
            audience = "my-project"
            poll_interval = "60s"

            [authentication.providers.jwt.header]
            name = "Authorization"
            value_prefix = "Bearer🎠 "
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 14, column 16
           |
        14 | value_prefix = "Bearer🎠 "
           |                ^^^^^^^^^^^^^
        invalid value: string "Bearer🎠 ", expected an ascii string
        "###);
    }

    #[test]
    fn telemetry() {
        // prepare
        let telemetry_config = TelemetryConfig {
            service_name: "test".to_string(),
            resource_attributes: Default::default(),
            tracing: Default::default(),
            exporters: Default::default(),
            logs: Default::default(),
            metrics: Default::default(),
            grafbase: Default::default(),
        };

        let input = indoc! {r#"
            [telemetry]
            service_name = "test"
        "#};

        // act
        let config: Config = toml::from_str(input).unwrap();

        // assert
        assert_eq!(telemetry_config, config.telemetry);
    }

    #[test]
    fn header_rename_duplicate() {
        let input = indoc! {r#"
            [[headers]]
            rule = "rename_duplicate"
            name = "content-type"
            default = "foo"
            rename = "something"
        "#};

        let result: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&result.headers, @r###"
        [
            RenameDuplicate(
                RenameDuplicate {
                    name: DynamicString(
                        "content-type",
                    ),
                    default: Some(
                        DynamicString(
                            "foo",
                        ),
                    ),
                    rename: DynamicString(
                        "something",
                    ),
                },
            ),
        ]
        "###);
    }

    #[test]
    fn header_forward_static() {
        let input = indoc! {r#"
            [[headers]]
            rule = "forward"
            name = "content-type"
        "#};

        let result: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&result.headers, @r###"
        [
            Forward(
                HeaderForward {
                    name: Name(
                        DynamicString(
                            "content-type",
                        ),
                    ),
                    default: None,
                    rename: None,
                },
            ),
        ]
        "###);
    }

    #[test]
    fn header_forward_invalid_name() {
        let input = indoc! {r#"
            [[headers]]
            rule = "forward"
            name = "Authoriz🎠"
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 1, column 1
          |
        1 | [[headers]]
          | ^^^^^^^^^^^
        the byte at index 8 is not ASCII
        "###);
    }

    #[test]
    fn header_forward_two_headers_in_written_order() {
        let input = indoc! {r#"
            [[headers]]
            rule = "forward"
            name = "content-type"

            [[headers]]
            rule = "forward"
            name = "accept"
        "#};

        let result: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&result.headers, @r###"
        [
            Forward(
                HeaderForward {
                    name: Name(
                        DynamicString(
                            "content-type",
                        ),
                    ),
                    default: None,
                    rename: None,
                },
            ),
            Forward(
                HeaderForward {
                    name: Name(
                        DynamicString(
                            "accept",
                        ),
                    ),
                    default: None,
                    rename: None,
                },
            ),
        ]
        "###);
    }

    #[test]
    fn header_forward_pattern() {
        let input = indoc! {r#"
            [[headers]]
            rule = "forward"
            pattern = "^content-type-*"
        "#};

        let result: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&result.headers, @r###"
        [
            Forward(
                HeaderForward {
                    name: Pattern(
                        Regex(
                            "^content-type-*",
                        ),
                    ),
                    default: None,
                    rename: None,
                },
            ),
        ]
        "###);
    }

    #[test]
    fn header_forward_invalid_pattern() {
        let input = indoc! {r#"
            [[headers]]
            rule = "forward"
            pattern = "foo(bar"
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 1, column 1
          |
        1 | [[headers]]
          | ^^^^^^^^^^^
        regex parse error:
            foo(bar
               ^
        error: unclosed group
        "###);
    }

    #[test]
    fn header_forward_default() {
        let input = indoc! {r#"
            [[headers]]
            rule = "forward"
            name = "content-type"
            default = "application/json"
        "#};

        let result: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&result.headers, @r###"
        [
            Forward(
                HeaderForward {
                    name: Name(
                        DynamicString(
                            "content-type",
                        ),
                    ),
                    default: Some(
                        DynamicString(
                            "application/json",
                        ),
                    ),
                    rename: None,
                },
            ),
        ]
        "###);
    }

    #[test]
    fn header_forward_invalid_default() {
        let input = indoc! {r#"
            [[headers]]
            rule = "forward"
            name = "content-type"
            default = "application/json🎠"
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 1, column 1
          |
        1 | [[headers]]
          | ^^^^^^^^^^^
        the byte at index 16 is not ASCII
        "###);
    }

    #[test]
    fn header_forward_rename() {
        let input = indoc! {r#"
            [[headers]]
            rule = "forward"
            name = "content-type"
            rename = "kekw-type"
        "#};

        let result: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&result.headers, @r###"
        [
            Forward(
                HeaderForward {
                    name: Name(
                        DynamicString(
                            "content-type",
                        ),
                    ),
                    default: None,
                    rename: Some(
                        DynamicString(
                            "kekw-type",
                        ),
                    ),
                },
            ),
        ]
        "###);
    }

    #[test]
    fn header_forward_invalid_rename() {
        let input = indoc! {r#"
            [[headers]]
            rule = "forward"
            name = "content-type"
            rename = "🎠"
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 1, column 1
          |
        1 | [[headers]]
          | ^^^^^^^^^^^
        the byte at index 0 is not ASCII
        "###);
    }

    #[test]
    fn header_insert() {
        let input = indoc! {r#"
            [[headers]]
            rule = "insert"
            name = "content-type"
            value = "application/json"
        "#};

        let result: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&result.headers, @r###"
        [
            Insert(
                HeaderInsert {
                    name: DynamicString(
                        "content-type",
                    ),
                    value: DynamicString(
                        "application/json",
                    ),
                },
            ),
        ]
        "###);
    }

    #[test]
    fn header_insert_env() {
        temp_env::with_var("CONTENT_TYPE", Some("application/json"), || {
            let input = indoc! {r#"
                [[headers]]
                rule = "insert"
                name = "content-type"
                value = "{{ env.CONTENT_TYPE }}"
            "#};

            let result: Config = toml::from_str(input).unwrap();

            insta::assert_debug_snapshot!(&result.headers, @r###"
            [
                Insert(
                    HeaderInsert {
                        name: DynamicString(
                            "content-type",
                        ),
                        value: DynamicString(
                            "application/json",
                        ),
                    },
                ),
            ]
            "###);
        })
    }

    #[test]
    fn header_insert_invalid_name() {
        let input = indoc! {r#"
            [[headers]]
            rule = "insert"
            name = "content-type🎠"
            value = "application/json"
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 1, column 1
          |
        1 | [[headers]]
          | ^^^^^^^^^^^
        the byte at index 12 is not ASCII
        "###);
    }

    #[test]
    fn header_insert_invalid_value() {
        let input = indoc! {r#"
            [[headers]]
            rule = "insert"
            name = "content-type"
            value = "application/json🎠"
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 1, column 1
          |
        1 | [[headers]]
          | ^^^^^^^^^^^
        the byte at index 16 is not ASCII
        "###);
    }

    #[test]
    fn header_remove() {
        let input = indoc! {r#"
            [[headers]]
            rule = "remove"
            name = "content-type"
        "#};

        let result: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&result.headers, @r###"
        [
            Remove(
                HeaderRemove {
                    name: Name(
                        DynamicString(
                            "content-type",
                        ),
                    ),
                },
            ),
        ]
        "###);
    }

    #[test]
    fn header_remove_invalid_name() {
        let input = indoc! {r#"
            [[headers]]
            rule = "remove"
            name = "content-type🎠"
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 1, column 1
          |
        1 | [[headers]]
          | ^^^^^^^^^^^
        the byte at index 12 is not ASCII
        "###);
    }

    #[test]
    fn subgraph_header_forward_static() {
        let input = indoc! {r#"
            [[subgraphs.products.headers]]
            rule = "forward"
            name = "content-type"
        "#};

        let result: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&result.subgraphs, @r#"
        {
            "products": SubgraphConfig {
                url: None,
                headers: [
                    Forward(
                        HeaderForward {
                            name: Name(
                                DynamicString(
                                    "content-type",
                                ),
                            ),
                            default: None,
                            rename: None,
                        },
                    ),
                ],
                websocket_url: None,
                rate_limit: None,
                timeout: None,
                retry: None,
                entity_caching: None,
            },
        }
        "#);
    }

    #[test]
    fn subgraph_ws_valid_url() {
        let input = indoc! {r#"
            [subgraphs.products]
            websocket_url = "https://example.com"
        "#};

        let result: Config = toml::from_str(input).unwrap();
        let subgraph = result.subgraphs.get("products").unwrap();

        insta::assert_debug_snapshot!(&subgraph.websocket_url.as_ref().map(|u| u.to_string()), @r###"
        Some(
            "https://example.com/",
        )
        "###);
    }

    #[test]
    fn subgraph_ws_invalid_url() {
        let input = indoc! {r#"
            [subgraphs.products]
            websocket_url = "WRONG"
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 2, column 17
          |
        2 | websocket_url = "WRONG"
          |                 ^^^^^^^
        invalid value: string "WRONG", expected relative URL without a base
        "###);
    }

    #[test]
    fn timeouts() {
        let input = indoc! {r#"
            [gateway]
            timeout = "1s"
            subgraph_timeout = "2s"
        "#};

        let config: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway, @r#"
        GatewayConfig {
            timeout: Some(
                1s,
            ),
            subgraph_timeout: Some(
                2s,
            ),
            rate_limit: None,
            retry: RetryConfig {
                enabled: false,
                min_per_second: None,
                ttl: None,
                retry_percent: None,
                retry_mutations: false,
            },
            access_logs: AccessLogsConfig {
                enabled: false,
                path: "",
                rotate: Never,
                mode: Blocking,
            },
            batching: BatchingConfig {
                enabled: false,
                limit: None,
            },
        }
        "#);
    }

    #[test]
    fn global_rate_limiting() {
        let input = indoc! {r#"
            [gateway.rate_limit.global]
            limit = 1000
            duration = "10s"
        "#};

        let config = toml::from_str::<Config>(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.rate_limit, @r###"
        Some(
            RateLimitConfig {
                global: Some(
                    GraphRateLimit {
                        limit: 1000,
                        duration: 10s,
                    },
                ),
                storage: Memory,
                redis: RateLimitRedisConfig {
                    url: Url {
                        scheme: "redis",
                        cannot_be_a_base: false,
                        username: "",
                        password: None,
                        host: Some(
                            Domain(
                                "localhost",
                            ),
                        ),
                        port: Some(
                            6379,
                        ),
                        path: "",
                        query: None,
                        fragment: None,
                    },
                    key_prefix: "grafbase",
                    tls: None,
                },
            },
        )
        "###);
    }

    #[test]
    fn global_rate_limiting_redis_defaults() {
        let input = indoc! {r#"
            [gateway.rate_limit]
            storage = "redis"
        "#};

        let config = toml::from_str::<Config>(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.rate_limit, @r###"
        Some(
            RateLimitConfig {
                global: None,
                storage: Redis,
                redis: RateLimitRedisConfig {
                    url: Url {
                        scheme: "redis",
                        cannot_be_a_base: false,
                        username: "",
                        password: None,
                        host: Some(
                            Domain(
                                "localhost",
                            ),
                        ),
                        port: Some(
                            6379,
                        ),
                        path: "",
                        query: None,
                        fragment: None,
                    },
                    key_prefix: "grafbase",
                    tls: None,
                },
            },
        )
        "###);
    }

    #[test]
    fn global_rate_limiting_redis_custom_url() {
        let input = indoc! {r#"
            [gateway.rate_limit]
            storage = "redis"

            [gateway.rate_limit.redis]
            url = "redis://user:password@localhost:420"
        "#};

        let config = toml::from_str::<Config>(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.rate_limit, @r###"
        Some(
            RateLimitConfig {
                global: None,
                storage: Redis,
                redis: RateLimitRedisConfig {
                    url: Url {
                        scheme: "redis",
                        cannot_be_a_base: false,
                        username: "user",
                        password: Some(
                            "password",
                        ),
                        host: Some(
                            Domain(
                                "localhost",
                            ),
                        ),
                        port: Some(
                            420,
                        ),
                        path: "",
                        query: None,
                        fragment: None,
                    },
                    key_prefix: "grafbase",
                    tls: None,
                },
            },
        )
        "###);
    }

    #[test]
    fn global_rate_limiting_redis_custom_key_prefix() {
        let input = indoc! {r#"
            [gateway.rate_limit]
            storage = "redis"

            [gateway.rate_limit.redis]
            key_prefix = "kekw"
        "#};

        let config = toml::from_str::<Config>(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.rate_limit, @r###"
        Some(
            RateLimitConfig {
                global: None,
                storage: Redis,
                redis: RateLimitRedisConfig {
                    url: Url {
                        scheme: "redis",
                        cannot_be_a_base: false,
                        username: "",
                        password: None,
                        host: Some(
                            Domain(
                                "localhost",
                            ),
                        ),
                        port: Some(
                            6379,
                        ),
                        path: "",
                        query: None,
                        fragment: None,
                    },
                    key_prefix: "kekw",
                    tls: None,
                },
            },
        )
        "###);
    }

    #[test]
    fn global_rate_limiting_redis_tls() {
        let input = indoc! {r#"
            [gateway.rate_limit]
            storage = "redis"

            [gateway.rate_limit.redis.tls]
            cert = "/path/to/cert.pem"
            key = "/path/to/key.pem"
        "#};

        let config = toml::from_str::<Config>(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.rate_limit, @r###"
        Some(
            RateLimitConfig {
                global: None,
                storage: Redis,
                redis: RateLimitRedisConfig {
                    url: Url {
                        scheme: "redis",
                        cannot_be_a_base: false,
                        username: "",
                        password: None,
                        host: Some(
                            Domain(
                                "localhost",
                            ),
                        ),
                        port: Some(
                            6379,
                        ),
                        path: "",
                        query: None,
                        fragment: None,
                    },
                    key_prefix: "grafbase",
                    tls: Some(
                        RateLimitRedisTlsConfig {
                            cert: Some(
                                "/path/to/cert.pem",
                            ),
                            key: Some(
                                "/path/to/key.pem",
                            ),
                            ca: None,
                        },
                    ),
                },
            },
        )
        "###);
    }

    #[test]
    fn global_rate_limiting_redis_tls_custom_ca() {
        let input = indoc! {r#"
            [gateway.rate_limit]
            storage = "redis"

            [gateway.rate_limit.redis.tls]
            cert = "/path/to/cert.pem"
            key = "/path/to/key.pem"
            ca = "ca.crt"
        "#};

        let config = toml::from_str::<Config>(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.rate_limit, @r###"
        Some(
            RateLimitConfig {
                global: None,
                storage: Redis,
                redis: RateLimitRedisConfig {
                    url: Url {
                        scheme: "redis",
                        cannot_be_a_base: false,
                        username: "",
                        password: None,
                        host: Some(
                            Domain(
                                "localhost",
                            ),
                        ),
                        port: Some(
                            6379,
                        ),
                        path: "",
                        query: None,
                        fragment: None,
                    },
                    key_prefix: "grafbase",
                    tls: Some(
                        RateLimitRedisTlsConfig {
                            cert: Some(
                                "/path/to/cert.pem",
                            ),
                            key: Some(
                                "/path/to/key.pem",
                            ),
                            ca: Some(
                                "ca.crt",
                            ),
                        },
                    ),
                },
            },
        )
        "###);
    }

    #[test]
    fn subgraph_rate_limiting() {
        let input = indoc! {r#"
            [subgraphs.products.rate_limit]
            limit = 1000
            duration = "10s"
        "#};

        let config = toml::from_str::<Config>(input).unwrap();

        assert!(config.gateway.rate_limit.is_none());
        insta::assert_debug_snapshot!(&config.subgraphs.get("products").unwrap().rate_limit, @r###"
        Some(
            GraphRateLimit {
                limit: 1000,
                duration: 10s,
            },
        )
        "###);
    }

    #[test]
    fn rate_limiting_invalid_duration() {
        let input = indoc! {r#"
            [subgraphs.products.rate_limit]
            limit = 1000
            duration = "0s"
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_debug_snapshot!(&error.to_string(), @r###""TOML parse error at line 3, column 12\n  |\n3 | duration = \"0s\"\n  |            ^^^^\nrate limit duration cannot be 0\n""###);
    }

    #[test]
    fn subgraph_global_retry() {
        let input = indoc! {r#"
            [gateway.retry]
            enabled = true
        "#};

        let config: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.retry, @r###"
        RetryConfig {
            enabled: true,
            min_per_second: None,
            ttl: None,
            retry_percent: None,
            retry_mutations: false,
        }
        "###);
    }

    #[test]
    fn subgraph_retry() {
        let input = indoc! {r#"
            [subgraphs.products.retry]
            enabled = true
        "#};

        let config: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&config.subgraphs, @r#"
        {
            "products": SubgraphConfig {
                url: None,
                headers: [],
                websocket_url: None,
                rate_limit: None,
                timeout: None,
                retry: Some(
                    RetryConfig {
                        enabled: true,
                        min_per_second: None,
                        ttl: None,
                        retry_percent: None,
                        retry_mutations: false,
                    },
                ),
                entity_caching: None,
            },
        }
        "#);
    }

    #[test]
    fn access_logs_default() {
        let input = indoc! {r#"
            [gateway.access_logs]
            enabled = true
            path = "/path"
        "#};

        let config: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.access_logs, @r###"
        AccessLogsConfig {
            enabled: true,
            path: "/path",
            rotate: Never,
            mode: Blocking,
        }
        "###);
    }

    #[test]
    fn access_logs_rotate_minutely() {
        let input = indoc! {r#"
            [gateway.access_logs]
            enabled = true
            path = "/path"
            rotate = "minutely"
        "#};

        let config: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.access_logs, @r###"
        AccessLogsConfig {
            enabled: true,
            path: "/path",
            rotate: Minutely,
            mode: Blocking,
        }
        "###);
    }

    #[test]
    fn access_logs_rotate_max_size() {
        let input = indoc! {r#"
            [gateway.access_logs]
            enabled = true
            path = "/path"
            rotate.size = 1024
        "#};

        let config: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.access_logs, @r###"
        AccessLogsConfig {
            enabled: true,
            path: "/path",
            rotate: Size(
                1024 bytes,
            ),
            mode: Blocking,
        }
        "###);

        let input = indoc! {r#"
            [gateway.access_logs]
            enabled = true
            path = "/path"
            rotate.size = "1kiB"
        "#};

        let config: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.access_logs, @r###"
        AccessLogsConfig {
            enabled: true,
            path: "/path",
            rotate: Size(
                1024 bytes,
            ),
            mode: Blocking,
        }
        "###);
    }

    #[test]
    fn batching_default() {
        let input = indoc! {r#"
        "#};

        let config: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.batching, @r#"
        BatchingConfig {
            enabled: false,
            limit: None,
        }
        "#);
    }

    #[test]
    fn batching_with_limit() {
        let input = indoc! {r#"
            [gateway.batching]
            enabled = true
            limit = 5
        "#};

        let config: Config = toml::from_str(input).unwrap();

        insta::assert_debug_snapshot!(&config.gateway.batching, @r#"
        BatchingConfig {
            enabled: true,
            limit: Some(
                5,
            ),
        }
        "#);
    }

    #[test]
    fn batching_with_too_high_limit() {
        let input = indoc! {r#"
            [gateway.batching]
            enabled = true
            limit = 1000
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r#"
        TOML parse error at line 3, column 9
          |
        3 | limit = 1000
          |         ^^^^
        invalid value: integer `1000`, expected u8
        "#);
    }
}
