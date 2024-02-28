#![allow(dead_code)] // TODO: remove when we use the configuration
#![allow(unused_imports)] // TODO: remove when we use the configuration

mod cors;

use std::{net::SocketAddr, path::PathBuf};

pub use cors::{AnyOrAsciiStringArray, AnyOrHttpMethodArray, AnyOrUrlArray, CorsConfig, HttpMethod};

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

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationLimitsConfig {
    /// Limits the deepest nesting of selection sets in an operation,
    /// including fields in fragments.
    pub depth: u16,
    /// Limits the number of unique fields included in an operation,
    /// including fields of fragments. If a particular field is included
    /// multiple times via aliases, it's counted only once.
    pub height: u16,
    /// Limits the total number of aliased fields in an operation,
    /// including fields of fragments.
    pub aliases: u16,
    /// Limits the number of root fields in an operation, including root
    /// fields in fragments. If a particular root field is included multiple
    /// times via aliases, each usage is counted.
    pub root_fields: u16,
    /// Query complexity takes the number of fields as well as the depth and
    /// any pagination arguments into account. Every scalar field adds 1 point,
    /// every nested field adds 2 points, and every pagination argument multiplies
    /// the nested objects score by the number of records fetched.
    pub complexity: u16,
}

#[cfg(test)]
mod tests {
    use crate::start::configuration::AnyOrAsciiStringArray;
    use crate::start::configuration::AnyOrHttpMethodArray;
    use crate::start::configuration::HttpMethod;

    use super::AnyOrUrlArray;
    use super::Config;
    use super::OperationLimitsConfig;
    use ascii::AsciiString;
    use indoc::indoc;
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
    use url::Url;

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
            depth: 3,
            height: 10,
            aliases: 100,
            root_fields: 10,
            complexity: 1000,
        };

        assert_eq!(expected, operation_limits);
    }

    #[test]
    fn operation_limits_with_missing_values() {
        let input = indoc! {r#"
            [operation_limits]
            depth = 3
            height = 10
        "#};

        let error = toml::from_str::<Config>(input).unwrap_err();

        insta::assert_snapshot!(&error.to_string(), @r###"
        TOML parse error at line 1, column 1
          |
        1 | [operation_limits]
          | ^^^^^^^^^^^^^^^^^^
        missing field `aliases`
        "###);
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
}
