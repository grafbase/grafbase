use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OperationCacheConfig {
    /// If operation caching should be enabled.
    pub enabled: bool,
    /// The maximum number of operations that can be kept in the cache.
    /// 1000 by default.
    pub limit: usize,
    /// Whether the cache should be warmed before schema/config reload
    pub warm_on_reload: bool,
    /// The percentage of the cache that will be warmed on reload
    pub warming_percent: u8,

    /// Configuration for a redis server that will be used as a fallback if
    /// in memory cache misses
    pub redis: Option<OperationCachingRedisConfig>,
}

impl Default for OperationCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            limit: 1000,
            warm_on_reload: false,
            warming_percent: 100,
            redis: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OperationCachingRedisConfig {
    pub url: url::Url,
    pub key_prefix: String,
    pub tls: Option<OperationCachingRedisTlsConfig>,
}

impl Default for OperationCachingRedisConfig {
    fn default() -> Self {
        Self {
            url: url::Url::parse("redis://localhost:6379").expect("must be correct"),
            key_prefix: String::from("grafbase-operation-cache"),
            tls: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationCachingRedisTlsConfig {
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub ca: Option<PathBuf>,
}
