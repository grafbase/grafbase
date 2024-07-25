use duration_str::deserialize_duration;
use serde::de::Error;
use serde::Deserializer;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, serde::Deserialize)]
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

impl From<RateLimitConfig> for parser_sdl::federation::RateLimitConfig {
    fn from(value: RateLimitConfig) -> Self {
        Self {
            global: value.global.map(Into::into),
            storage: value.storage.into(),
            redis: value.redis.into(),
        }
    }
}

impl From<GraphRateLimit> for parser_sdl::federation::GraphRateLimit {
    fn from(value: GraphRateLimit) -> Self {
        Self {
            limit: value.limit,
            duration: value.duration,
        }
    }
}

impl From<RateLimitStorage> for parser_sdl::federation::RateLimitStorage {
    fn from(value: RateLimitStorage) -> Self {
        match value {
            RateLimitStorage::Memory => Self::Memory,
            RateLimitStorage::Redis => Self::Redis,
        }
    }
}

impl From<RateLimitRedisConfig> for parser_sdl::federation::RateLimitRedisConfig {
    fn from(value: RateLimitRedisConfig) -> Self {
        Self {
            url: value.url,
            key_prefix: value.key_prefix,
            tls: value.tls.map(Into::into),
        }
    }
}

impl From<RateLimitRedisTlsConfig> for parser_sdl::federation::RateLimitRedisTlsConfig {
    fn from(value: RateLimitRedisTlsConfig) -> Self {
        Self {
            cert: value.cert,
            key: value.key,
            ca: value.ca,
        }
    }
}
