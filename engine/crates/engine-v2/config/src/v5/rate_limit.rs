use std::{path::PathBuf, time::Duration};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitConfig {
    pub limit: usize,
    pub duration: Duration,
    #[serde(default)]
    pub storage: RateLimitStorage,
    #[serde(default)]
    pub redis: RateLimitRedisConfig,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum RateLimitStorage {
    #[default]
    InMemory,
    Redis,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitRedisConfig {
    #[serde(default = "RateLimitRedisConfig::default_url")]
    pub url: url::Url,
    #[serde(default = "RateLimitRedisConfig::default_key_prefix")]
    pub key_prefix: String,
    #[serde(default)]
    pub tls: Option<RateLimitRedisTlsConfig>,
}

impl Default for RateLimitRedisConfig {
    fn default() -> Self {
        Self {
            url: Self::default_url(),
            key_prefix: Self::default_key_prefix(),
            tls: None,
        }
    }
}

impl RateLimitRedisConfig {
    fn default_url() -> url::Url {
        url::Url::parse("redis://localhost:6379").expect("must be correct")
    }

    fn default_key_prefix() -> String {
        String::from("grafbase")
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitRedisTlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
    pub ca: Option<PathBuf>,
}
