use std::{path::PathBuf, time::Duration};

const DEFAULT_ENTITY_CACHE_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Default, serde::Deserialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct SubgraphEntityCachingConfig {
    /// Defaults to global entity cache value.
    pub enabled: Option<bool>,
    /// The ttl to store cache entries with. Defaults to global entity cache TTL value
    #[serde(deserialize_with = "duration_str::deserialize_option_duration")]
    pub ttl: Option<Duration>,
}

#[derive(Debug, serde::Deserialize, Clone, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct EntityCachingConfig {
    pub enabled: bool,
    pub storage: EntityCachingStorage,
    pub redis: EntityCachingRedisConfig,

    /// The ttl to store cache entries with.  Defaults to 60s
    #[serde(deserialize_with = "duration_str::deserialize_duration")]
    pub ttl: Duration,
}

impl Default for EntityCachingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            storage: Default::default(),
            redis: Default::default(),
            ttl: DEFAULT_ENTITY_CACHE_TTL,
        }
    }
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
