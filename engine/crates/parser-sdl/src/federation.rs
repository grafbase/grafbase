pub mod header;

use std::time::Duration;
use std::{collections::BTreeMap, path::PathBuf};

use crate::{rules::auth_directive::v2::AuthV2Directive, GlobalCacheRules};
use registry_v2::{ConnectorHeaderValue, OperationLimits};

use self::header::{NameOrPattern, SubgraphHeaderForward, SubgraphHeaderInsert, SubgraphHeaderRule};

/// Configuration for a federated graph
#[derive(Clone, Debug, Default)]
pub struct FederatedGraphConfig {
    pub subgraphs: BTreeMap<String, SubgraphConfig>,
    pub header_rules: Vec<SubgraphHeaderRule>,
    pub operation_limits: OperationLimits,
    pub global_cache_rules: GlobalCacheRules<'static>,
    pub auth: Option<AuthV2Directive>,
    pub disable_introspection: bool,
    pub rate_limit: Option<RateLimitConfig>,
    pub timeout: Option<Duration>,
    pub entity_caching: EntityCachingConfig,
    pub retry: Option<RetryConfig>,
}

/// Configuration for a subgraph of the current federated graph
#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct SubgraphConfig {
    /// The name of the subgrah
    pub name: String,

    /// The URL to use in development
    ///
    /// This is only used in development and should be ignored in deployed
    /// environments
    pub development_url: Option<String>,

    /// The URL to use for GraphQL-WS calls.
    ///
    /// This will default to the normal URL if not present.
    pub websocket_url: Option<String>,

    /// Rules for passing headers forward to the subgraph
    pub header_rules: Vec<SubgraphHeaderRule>,

    /// Configuration to enforce rate limiting on subgraph requests
    pub rate_limit: Option<GraphRateLimit>,

    /// Timeouts to apply to subgraph requests
    pub timeout: Option<Duration>,

    /// Retry configuration
    pub retry: Option<RetryConfig>,

    /// Optional entity caching config for this subgraph.
    pub entity_caching: Option<EntityCachingConfig>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntityCachingConfig {
    #[default]
    Disabled,
    Enabled {
        ttl: Option<Duration>,
        storage: EntityCacheStorage,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum EntityCacheStorage {
    #[default]
    Memory,
    Redis(Box<RedisConfig>),
}

impl From<gateway_config::EntityCachingConfig> for EntityCachingConfig {
    fn from(config: gateway_config::EntityCachingConfig) -> Self {
        match (config.enabled, config.ttl) {
            (Some(false), _) => EntityCachingConfig::Disabled,
            (Some(true), ttl) => EntityCachingConfig::Enabled {
                ttl,
                storage: entity_cache_storage(config.storage, config.redis),
            },
            (_, Some(ttl)) => EntityCachingConfig::Enabled {
                ttl: Some(ttl),
                storage: entity_cache_storage(config.storage, config.redis),
            },
            _ => EntityCachingConfig::Disabled,
        }
    }
}

fn entity_cache_storage(
    storage: gateway_config::EntityCachingStorage,
    redis: gateway_config::EntityCachingRedisConfig,
) -> EntityCacheStorage {
    match storage {
        gateway_config::EntityCachingStorage::Memory => EntityCacheStorage::Memory,
        gateway_config::EntityCachingStorage::Redis => EntityCacheStorage::Redis(Box::new(redis.into())),
    }
}

impl From<gateway_config::EntityCachingRedisConfig> for RedisConfig {
    fn from(value: gateway_config::EntityCachingRedisConfig) -> Self {
        let gateway_config::EntityCachingRedisConfig { url, key_prefix, tls } = value;
        RedisConfig {
            url,
            key_prefix,
            tls: tls.map(Into::into),
        }
    }
}

impl From<gateway_config::EntityCachingRedisTlsConfig> for RedisTlsConfig {
    fn from(value: gateway_config::EntityCachingRedisTlsConfig) -> Self {
        let gateway_config::EntityCachingRedisTlsConfig { cert, key, ca } = value;
        RedisTlsConfig { cert, key, ca }
    }
}

impl From<(String, ConnectorHeaderValue)> for SubgraphHeaderRule {
    fn from((name, value): (String, ConnectorHeaderValue)) -> Self {
        match value {
            ConnectorHeaderValue::Static(value) => SubgraphHeaderRule::Insert(SubgraphHeaderInsert { name, value }),
            ConnectorHeaderValue::Forward(value) => SubgraphHeaderRule::Forward(SubgraphHeaderForward {
                name: NameOrPattern::Name(value),
                default: None,
                rename: Some(name),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphRateLimit {
    pub limit: usize,
    pub duration: Duration,
}

// we're simplifying federated rate limiting atm, taking the same config (registry_v2::rate_limiting::RateLimitConfig)
// for standalone v1 and local wouldn't work as its quite different
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RateLimitConfig {
    pub global: Option<GraphRateLimit>,
    pub storage: RateLimitStorage,
    pub redis: RedisConfig,
}

impl From<gateway_config::RateLimitConfig> for RateLimitConfig {
    fn from(value: gateway_config::RateLimitConfig) -> Self {
        Self {
            global: value.global.map(Into::into),
            storage: value.storage.into(),
            redis: value.redis.into(),
        }
    }
}

impl From<gateway_config::GraphRateLimit> for GraphRateLimit {
    fn from(value: gateway_config::GraphRateLimit) -> Self {
        Self {
            limit: value.limit,
            duration: value.duration,
        }
    }
}

impl From<gateway_config::RateLimitStorage> for RateLimitStorage {
    fn from(value: gateway_config::RateLimitStorage) -> Self {
        match value {
            gateway_config::RateLimitStorage::Memory => Self::Memory,
            gateway_config::RateLimitStorage::Redis => Self::Redis,
        }
    }
}

impl From<gateway_config::RateLimitRedisConfig> for RedisConfig {
    fn from(value: gateway_config::RateLimitRedisConfig) -> Self {
        Self {
            url: value.url,
            key_prefix: value.key_prefix,
            tls: value.tls.map(Into::into),
        }
    }
}

impl From<gateway_config::RateLimitRedisTlsConfig> for RedisTlsConfig {
    fn from(value: gateway_config::RateLimitRedisTlsConfig) -> Self {
        Self {
            cert: value.cert,
            key: value.key,
            ca: value.ca,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RateLimitStorage {
    Memory,
    Redis,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RedisConfig {
    pub url: url::Url,
    pub key_prefix: String,
    pub tls: Option<RedisTlsConfig>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RedisTlsConfig {
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub ca: Option<PathBuf>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct RetryConfig {
    /// How many retries are available per second, at a minimum.
    pub min_per_second: Option<u32>,
    /// Each successful request to the subgraph adds to the retry budget. This setting controls for how long the budget remembers successful requests.
    pub ttl: Option<Duration>,
    /// The fraction of the successful requests budget that can be used for retries.
    pub retry_percent: Option<f32>,
    /// Whether mutations should be retried at all. False by default.
    pub retry_mutations: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn entity_caching_global() {
        let input = indoc! {r#"
            [entity_caching]
            enabled = true
            ttl = "60s"
        "#};

        let config = toml::from_str::<gateway_config::Config>(input).unwrap();

        assert_eq!(
            EntityCachingConfig::from(config.entity_caching),
            EntityCachingConfig::Enabled {
                ttl: Some(Duration::from_secs(60)),
                storage: Default::default(),
            }
        )
    }

    #[test]
    fn entity_caching_subgraph() {
        let input = indoc! {r#"
            [subgraphs.products.entity_caching]
            ttl = "60s"
        "#};

        let mut config = toml::from_str::<gateway_config::Config>(input).unwrap();

        assert_eq!(
            EntityCachingConfig::from(config.subgraphs.remove("products").unwrap().entity_caching.unwrap()),
            EntityCachingConfig::Enabled {
                ttl: Some(Duration::from_secs(60)),
                storage: Default::default()
            }
        )
    }

    #[test]
    fn entity_caching_subgraph_enabled() {
        let input = indoc! {r#"
            [subgraphs.products.entity_caching]
            enabled = true
        "#};

        let mut config = toml::from_str::<gateway_config::Config>(input).unwrap();

        assert_eq!(
            EntityCachingConfig::from(config.subgraphs.remove("products").unwrap().entity_caching.unwrap()),
            EntityCachingConfig::Enabled {
                ttl: None,
                storage: Default::default()
            }
        )
    }

    #[test]
    fn entity_caching_subgraph_disabled() {
        let input = indoc! {r#"
            [subgraphs.products.entity_caching]
            enabled = false
            ttl = "60s"
        "#};

        let mut config = toml::from_str::<gateway_config::Config>(input).unwrap();

        assert_eq!(
            EntityCachingConfig::from(config.subgraphs.remove("products").unwrap().entity_caching.unwrap()),
            EntityCachingConfig::Disabled
        )
    }
}
