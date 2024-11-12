use std::{path::PathBuf, time::Duration};

#[derive(Debug, Default, serde::Deserialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct EntityCachingConfig {
    pub enabled: Option<bool>,
    pub storage: EntityCachingStorage,
    pub redis: EntityCachingRedisConfig,

    /// The ttl to store cache entries with.  Defaults to 60s
    #[serde(deserialize_with = "duration_str::deserialize_option_duration")]
    pub ttl: Option<Duration>,
}

#[derive(Debug, Clone, Default, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityCachingStorage {
    #[default]
    Memory,
    Redis,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EntityCachingRedisConfig {
    pub url: url::Url,
    pub key_prefix: String,
    pub tls: Option<EntityCachingRedisTlsConfig>,
}

impl Default for EntityCachingRedisConfig {
    fn default() -> Self {
        Self {
            url: url::Url::parse("redis://localhost:6379").expect("must be correct"),
            key_prefix: String::from("grafbase-cache"),
            tls: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityCachingRedisTlsConfig {
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub ca: Option<PathBuf>,
}
