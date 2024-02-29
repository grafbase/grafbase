use std::time::Duration;

use ascii::AsciiString;
use duration_str::deserialize_duration;
use parser_sdl::{AuthV2Directive, AuthV2Provider, Jwks, JwtTokenHeader};
use url::Url;

/// Configures the GraphQL server JWT authentication
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
pub struct AuthenticationConfig {
    /// Enabled authentication providers
    pub providers: Vec<AuthenticationProvider>,
}

#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationProvider {
    Jwt(JwtProvider),
}

#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
pub struct JwtProvider {
    /// A name of the provider, used for log/error messages
    pub name: Option<String>,
    /// The JWKS provider configuration
    pub jwks: JwksConfig,
    /// The header from which to look for the token
    #[serde(default)]
    pub header: AuthenticationHeader,
}

#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
pub struct JwksConfig {
    /// The well-known URL of the JWKS
    pub url: Url,
    /// The issuer URL
    pub issuer: Option<String>,
    /// The name of the audience, e.g. the project
    pub audience: Option<String>,
    /// How often to poll changes to the configuration
    #[serde(default = "default_poll_interval", deserialize_with = "deserialize_duration")]
    pub poll_interval: Duration,
}

fn default_poll_interval() -> Duration {
    Duration::from_secs(60)
}

#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
pub struct AuthenticationHeader {
    /// The name of the header the token is sent from
    pub name: AsciiString,
    /// The prefix of the header value, typically `Bearer `
    pub value_prefix: AsciiString,
}

impl Default for AuthenticationHeader {
    fn default() -> Self {
        Self {
            name: AsciiString::from_ascii(b"Authorization").expect("that is ascii"),
            value_prefix: AsciiString::from_ascii(b"Bearer ").expect("that is ascii"),
        }
    }
}

impl From<JwksConfig> for Jwks {
    fn from(value: JwksConfig) -> Self {
        Self {
            url: value.url,
            issuer: value.issuer,
            audience: value.audience,
            poll_interval: value.poll_interval,
        }
    }
}

impl From<AuthenticationProvider> for AuthV2Provider {
    fn from(value: AuthenticationProvider) -> Self {
        match value {
            AuthenticationProvider::Jwt(jwt) => Self::JWT {
                name: jwt.name,
                jwks: Jwks::from(jwt.jwks),
                header: JwtTokenHeader::from(jwt.header),
            },
        }
    }
}

impl From<AuthenticationHeader> for JwtTokenHeader {
    fn from(value: AuthenticationHeader) -> Self {
        Self {
            name: value.name.to_string(),
            value_prefix: value.value_prefix.to_string(),
        }
    }
}

impl From<AuthenticationConfig> for AuthV2Directive {
    fn from(value: AuthenticationConfig) -> Self {
        let providers = value.providers.into_iter().map(AuthV2Provider::from).collect();
        Self { providers }
    }
}
