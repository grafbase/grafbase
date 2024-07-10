use serde::{Deserialize, Serialize};

#[derive(Default, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    pub providers: Vec<AuthProviderConfig>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum AuthProviderConfig {
    Jwt(JwtConfig),
    Anonymous,
}

/// Basically whatever Apollo 'JWT Authentication' is doing.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct JwtConfig {
    /// Used for logging/error messages.
    pub name: Option<String>,
    pub jwks: JwksConfig,
    pub header_name: String,
    pub header_value_prefix: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct JwksConfig {
    pub issuer: Option<String>,
    pub audience: Option<String>,
    pub url: url::Url,
    pub poll_interval: std::time::Duration,
}
