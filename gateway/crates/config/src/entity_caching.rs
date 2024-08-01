use std::{path::PathBuf, time::Duration};

#[derive(Debug, Default, serde::Deserialize, Clone, PartialEq)]
pub struct EntityCachingConfig {
    pub enabled: Option<bool>,

    #[serde(default)]
    pub storage: EntityCachingStorage,

    #[serde(default)]
    pub redis: Option<EntityCachingRedisConfig>,

    /// The ttl to store cache entries with.  Defaults to 60s
    #[serde(deserialize_with = "duration_str::deserialize_option_duration", default)]
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
#[serde(deny_unknown_fields)]
pub struct EntityCachingRedisConfig {
    #[serde(default = "EntityCachingRedisConfig::default_url")]
    pub url: url::Url,
    #[serde(default = "EntityCachingRedisConfig::default_key_prefix")]
    pub key_prefix: String,
    pub tls: Option<EntityCachingRedisTlsConfig>,
}

impl Default for EntityCachingRedisConfig {
    fn default() -> Self {
        Self {
            url: Self::default_url(),
            key_prefix: Self::default_key_prefix(),
            tls: None,
        }
    }
}

impl EntityCachingRedisConfig {
    fn default_url() -> url::Url {
        url::Url::parse("redis://localhost:6379").expect("must be correct")
    }

    fn default_key_prefix() -> String {
        String::from("grafbase-cache")
    }
}

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityCachingRedisTlsConfig {
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub ca: Option<PathBuf>,
}
