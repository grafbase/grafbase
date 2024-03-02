use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub providers: Vec<AuthProviderConfig>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Serialize, Deserialize)]
pub enum AuthProviderConfig {
    Jwt(JwtConfig),
}

/// Basically whatever Apollo 'JWT Authentication' is doing.
#[derive(Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Used for logging/error messages.
    pub name: Option<String>,
    pub jwks: JwksConfig,
    pub header_name: String,
    pub header_value_prefix: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct JwksConfig {
    pub issuer: Option<String>,
    pub audience: Option<String>,
    pub url: url::Url,
    pub poll_interval: std::time::Duration,
}
