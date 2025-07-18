use ascii::AsciiString;
use duration_str::deserialize_duration;
use serde::Deserialize;
use std::time::Duration;
use url::Url;

/// Configures the GraphQL server JWT authentication
#[derive(Default, Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct AuthenticationConfig {
    pub default: Option<DefaultAuthenticationBehavior>,
    pub protected_resources: AuthenticationResources,
    /// Enabled authentication providers
    pub providers: Vec<AuthenticationProvider>,
}

#[derive(Default, Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct AuthenticationResources {
    pub graphql: AuthenticationResourcesConfig,
    pub mcp: AuthenticationResourcesConfig,
}

#[derive(Default, Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct AuthenticationResourcesConfig {
    pub extensions: Option<Vec<String>>,
    pub default: Option<DefaultAuthenticationBehavior>,
}

#[derive(Debug, PartialEq, serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DefaultAuthenticationBehavior {
    Anonymous,
    Deny,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, PartialEq, serde::Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationProvider {
    Jwt(JwtProvider),
    Anonymous,
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
    #[serde(deserialize_with = "deserialize_string_or_vec", default)]
    pub audience: Vec<String>,
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

// Add this helper function
fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> serde::de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("string or array of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![value.to_string()])
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![value])
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: serde::de::SeqAccess<'de>,
        {
            Deserialize::deserialize(serde::de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrVec)
}
