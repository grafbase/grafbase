use std::{borrow::Cow, net::SocketAddr};

/// Health endpoint configuration.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealthConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub listen: Option<SocketAddr>,
    #[serde(default = "default_path")]
    pub path: Cow<'static, str>,
}

fn default_path() -> Cow<'static, str> {
    Cow::Borrowed("/health")
}

fn default_true() -> bool {
    true
}

impl Default for HealthConfig {
    fn default() -> Self {
        HealthConfig {
            enabled: true,
            listen: None,
            path: default_path(),
        }
    }
}
