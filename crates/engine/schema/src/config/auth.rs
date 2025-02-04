use extension_catalog::{ExtensionCatalog, ExtensionId};
use serde::{Deserialize, Serialize};

#[derive(Default, PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct AuthConfig {
    pub providers: Vec<AuthProviderConfig>,
}

impl AuthConfig {
    pub fn new(auth: &gateway_config::AuthenticationConfig, extension_catalog: &ExtensionCatalog) -> Self {
        let mut providers = Vec::new();

        for provider in &auth.providers {
            match provider {
                gateway_config::AuthenticationProvider::Jwt(provider) => {
                    providers.push(AuthProviderConfig::Jwt(JwtConfig {
                        name: provider.name.clone(),
                        jwks: JwksConfig {
                            issuer: provider.jwks.issuer.clone(),
                            audience: provider.jwks.audience.clone(),
                            url: provider.jwks.url.clone(),
                            poll_interval: provider.jwks.poll_interval,
                        },
                        header_name: provider.header.name.to_string(),
                        header_value_prefix: provider.header.value_prefix.to_string(),
                    }));
                }
                gateway_config::AuthenticationProvider::Anonymous => {
                    providers.push(AuthProviderConfig::Anonymous);
                }
                gateway_config::AuthenticationProvider::Extension(provider) => {
                    if let Some(extension_id) = extension_catalog.get_id_by_name(&provider.extension) {
                        providers.push(AuthProviderConfig::Extension(extension_id));
                    } else {
                        tracing::error!(
                            "extension '{}' needed as an auth provider not found",
                            provider.extension
                        );
                    }
                }
            }
        }

        AuthConfig { providers }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum AuthProviderConfig {
    Jwt(JwtConfig),
    Extension(ExtensionId),
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
