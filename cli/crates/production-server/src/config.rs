mod authentication;
mod cors;
mod telemetry;

use std::{net::SocketAddr, path::PathBuf};

pub use authentication::AuthenticationConfig;
pub use cors::CorsConfig;
pub use telemetry::TelemetryConfig;

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
/// Configuration struct to define settings for self-hosted
/// Grafbase gateway.
pub struct Config {
    /// Graph location and features, such as introspection
    #[serde(default)]
    pub graph: GraphConfig,
    /// Server bind settings
    #[serde(default)]
    pub network: NetworkConfig,
    /// Cross-site request forgery settings
    #[serde(default)]
    pub csrf: CsrfConfig,
    /// Cross-origin resource sharing settings
    pub cors: Option<CorsConfig>,
    /// Server TLS settings
    pub tls: Option<TlsConfig>,
    /// Graph operation limit settings
    pub operation_limits: Option<OperationLimitsConfig>,
    /// Telemetry settings
    pub telemetry: Option<TelemetryConfig>,
    /// Configuration for Trusted Documents.
    #[serde(default)]
    pub trusted_documents: TrustedDocumentsConfig,
    /// Authentication configuration
    pub authentication: Option<AuthenticationConfig>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphConfig {
    pub path: Option<String>,
    #[serde(default)]
    pub introspection: bool,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CsrfConfig {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkConfig {
    pub listen_address: Option<SocketAddr>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TlsConfig {
    pub certificate: PathBuf,
    pub key: PathBuf,
}

#[derive(Debug, PartialEq, serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TrustedDocumentsConfig {
    /// If true, the engine will only accept trusted document queries. Default: false.
    #[serde(default)]
    pub enabled: bool,
    /// Optional name of the header that can be set to bypass trusted documents enforcement, when `enabled = true`. Only meaningful in combination with `bypass_header_value`.
    #[serde(default)]
    pub bypass_header_name: Option<String>,
    /// Optional value of the `bypass_header_name` header that can be set to bypass trusted documents enforcement, when `enabled = true`. Only meaningful in combination with `bypass_header_name`.
    #[serde(default)]
    pub bypass_header_value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
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

impl From<OperationLimitsConfig> for engine::registry::OperationLimits {
    fn from(value: OperationLimitsConfig) -> Self {
        Self {
            depth: value.depth,
            height: value.height,
            aliases: value.aliases,
            root_fields: value.root_fields,
            complexity: value.complexity,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::cors::AnyOrAsciiStringArray;
    use crate::config::cors::AnyOrHttpMethodArray;
    use crate::config::cors::AnyOrUrlArray;
    use crate::config::cors::HttpMethod;

    use super::OperationLimitsConfig;
    use super::TrustedDocumentsConfig;
    use super::{Config, TelemetryConfig};
    use ascii::AsciiString;
    use indoc::indoc;
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
    use std::time::Duration;

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
        data did not match any variant of untagged enum AnyOrUrlArray
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
        data did not match any variant of untagged enum AnyOrHttpMethodArray
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
        data did not match any variant of untagged enum AnyOrAsciiStringArray
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
        data did not match any variant of untagged enum AnyOrAsciiStringArray
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

        let expected = TrustedDocumentsConfig::default();
        assert_eq!(config.trusted_documents, expected);
    }

    #[test]
    fn trusted_documents_just_enabled() {
        let input = indoc! {r#"
            [trusted_documents]
            enabled = true
        "#};

        let config = toml::from_str::<Config>(input).unwrap();

        let expected = TrustedDocumentsConfig {
            enabled: true,
            ..Default::default()
        };
        assert_eq!(config.trusted_documents, expected);
    }

    #[test]
    fn trusted_documents_all_settings() {
        let input = indoc! {r#"
            [trusted_documents]
            enabled = true # default: false
            bypass_header_name = "my-header-name" # default null
            bypass_header_value = "my-secret-value" # default null
        "#};

        let config = toml::from_str::<Config>(input).unwrap();

        let expected = TrustedDocumentsConfig {
            enabled: true,
            bypass_header_name: Some("my-header-name".into()),
            bypass_header_value: Some("my-secret-value".into()),
        };

        assert_eq!(config.trusted_documents, expected);
    }

    #[test]
    fn trusted_documents_unknown_setting() {
        let input = indoc! {r#"
            [trusted_documents]
            copacetic = false
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();
        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 2, column 1
          |
        2 | copacetic = false
          | ^^^^^^^^^
        unknown field `copacetic`, expected one of `enabled`, `bypass_header_name`, `bypass_header_value`
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
            tracing: Default::default(),
        };

        let input = indoc! {r#"
            [telemetry]
            service_name = "test"
        "#};

        // act
        let config: Config = toml::from_str(input).unwrap();

        // assert
        assert_eq!(telemetry_config, config.telemetry.unwrap());
    }
}
