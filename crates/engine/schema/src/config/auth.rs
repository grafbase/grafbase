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

impl From<&gateway_config::AuthenticationConfig> for AuthConfig {
    fn from(auth: &gateway_config::AuthenticationConfig) -> Self {
        let providers = auth
            .providers
            .iter()
            .map(|provider| match provider {
                gateway_config::AuthenticationProvider::Jwt(provider) => AuthProviderConfig::Jwt(JwtConfig {
                    name: provider.name.clone(),
                    jwks: JwksConfig {
                        issuer: provider.jwks.issuer.clone(),
                        audience: provider.jwks.audience.clone(),
                        url: provider.jwks.url.clone(),
                        poll_interval: provider.jwks.poll_interval,
                    },
                    header_name: provider.header.name.to_string(),
                    header_value_prefix: provider.header.value_prefix.to_string(),
                }),
                gateway_config::AuthenticationProvider::Anonymous => AuthProviderConfig::Anonymous,
            })
            .collect();

        AuthConfig { providers }
    }
}
