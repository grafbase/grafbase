pub mod header;

use std::time::Duration;
use std::{collections::BTreeMap, path::PathBuf};

use crate::{rules::auth_directive::v2::AuthV2Directive, GlobalCacheRules};
use registry_v2::{ConnectorHeaderValue, OperationLimits};

use self::header::{
    NameOrPattern, SubgraphHeaderForward, SubgraphHeaderInsert, SubgraphHeaderRule,
};

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
    },
}

impl From<(String, ConnectorHeaderValue)> for SubgraphHeaderRule {
    fn from((name, value): (String, ConnectorHeaderValue)) -> Self {
        match value {
            ConnectorHeaderValue::Static(value) => {
                SubgraphHeaderRule::Insert(SubgraphHeaderInsert { name, value })
            }
            ConnectorHeaderValue::Forward(value) => {
                SubgraphHeaderRule::Forward(SubgraphHeaderForward {
                    name: NameOrPattern::Name(value),
                    default: None,
                    rename: Some(name),
                })
            }
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
    pub redis: RateLimitRedisConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RateLimitStorage {
    Memory,
    Redis,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RateLimitRedisConfig {
    pub url: url::Url,
    pub key_prefix: String,
    pub tls: Option<RateLimitRedisTlsConfig>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RateLimitRedisTlsConfig {
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub ca: Option<PathBuf>,
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct RetryConfig {
    /// How many retries are available per second, at a minimum.
    pub min_per_second: Option<u32>,
    /// Each successful request to the subgraph adds to the retry budget. This setting controls for how long the budget remembers successful requests.
    pub ttl: Option<Duration>,
    /// The fraction of the successful requests budget that can be used for retries.
    pub retry_percent: Option<f32>,
    /// Whether mutations should be retried at all. False by default.
    pub retry_mutations: Option<bool>,
}
