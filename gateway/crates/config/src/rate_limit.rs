use duration_str::deserialize_duration;
use serde::de::Error;
use serde::Deserializer;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphRateLimit {
    pub limit: usize,
    #[serde(deserialize_with = "deserialize_duration_internal")]
    pub duration: Duration,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RateLimitConfig {
    pub global: Option<GraphRateLimit>,
    #[serde(default)]
    pub storage: RateLimitStorage,
    #[serde(default)]
    pub redis: RateLimitRedisConfig,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitStorage {
    #[default]
    Memory,
    Redis,
}

impl RateLimitStorage {
    pub fn is_redis(&self) -> bool {
        matches!(self, Self::Redis)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RateLimitRedisConfig {
    #[serde(default = "RateLimitRedisConfig::default_url")]
    pub url: url::Url,
    #[serde(default = "RateLimitRedisConfig::default_key_prefix")]
    pub key_prefix: String,
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

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RateLimitRedisTlsConfig {
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub ca: Option<PathBuf>,
}

fn deserialize_duration_internal<'de, D>(data: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let duration = deserialize_duration(data)?;

    if duration.as_secs() == 0 {
        return Err(Error::custom("rate limit duration cannot be 0"));
    }

    Ok(duration)
}
