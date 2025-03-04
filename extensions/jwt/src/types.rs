use duration_str::deserialize_duration;
use jwt_compact::jwk::JsonWebKey;
use std::{borrow::Cow, time::Duration};
use url::Url;

#[derive(Debug, serde::Deserialize)]
pub(super) struct JwtConfig {
    pub url: Url,
    pub issuer: Option<String>,
    pub audience: Option<String>,
    #[serde(default = "default_poll_interval", deserialize_with = "deserialize_duration")]
    pub poll_interval: Duration,
    header_name: Option<String>,
    header_value_prefix: Option<String>,
}

impl JwtConfig {
    pub fn header_name(&self) -> &str {
        self.header_name.as_deref().unwrap_or("Authorization")
    }

    pub fn header_value_prefix(&self) -> &str {
        self.header_value_prefix.as_deref().unwrap_or("Bearer ")
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct Jwks<'a> {
    pub keys: Vec<Jwk<'a>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct Jwk<'a> {
    #[serde(flatten)]
    pub key: JsonWebKey<'a>,
    #[serde(rename = "kid")]
    pub key_id: Option<Cow<'a, str>>,
}

#[serde_with::serde_as]
#[derive(Debug, serde::Deserialize)]
pub(super) struct CustomClaims {
    #[serde(default, rename = "iss")]
    pub issuer: Option<String>,
    #[serde_as(deserialize_as = "Option<serde_with::OneOrMany<_>>")]
    #[serde(default, rename = "aud")]
    pub audience: Option<Vec<String>>,
}

impl<'a> std::ops::Deref for Jwk<'a> {
    type Target = JsonWebKey<'a>;

    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

#[derive(Debug, strum::EnumString)]
pub(super) enum Alg {
    HS256,
    HS384,
    HS512,
    ES256,
    RS256,
    RS384,
    RS512,
    PS256,
    PS384,
    PS512,
    EdDSA,
}

fn default_poll_interval() -> Duration {
    Duration::from_secs(60)
}
