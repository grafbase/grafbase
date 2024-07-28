use std::time::Duration;

use ascii::AsciiString;
use duration_str::deserialize_duration;
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
