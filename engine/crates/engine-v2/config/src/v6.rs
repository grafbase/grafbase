use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

use federated_graph::{FederatedGraph, SubgraphId};

use crate::v5::{RateLimitConfigRef, RateLimitRedisConfigRef, RateLimitRedisTlsConfigRef};

pub use super::v5::{
    AuthConfig, AuthProviderConfig, CacheConfig, CacheConfigTarget, CacheConfigs, EntityCaching, GraphRateLimit,
    Header, HeaderForward, HeaderInsert, HeaderRemove, HeaderRenameDuplicate, HeaderRule, HeaderRuleId, HeaderValue,
    JwksConfig, JwtConfig, NameOrPattern, OperationLimits, PathId, RateLimitConfig, RateLimitKey, RateLimitRedisConfig,
    RateLimitRedisTlsConfig, RateLimitStorage, RetryConfig, StringId, SubgraphConfig,
};

/// Configuration for a federated graph
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Config {
    #[serde(
        serialize_with = "serialize_to_federated_sdl",
        deserialize_with = "deserialize_from_federated_sdl",
        rename = "federated_sdl"
    )]
    pub graph: FederatedGraph,
    pub strings: Vec<String>,
    #[serde(default)]
    pub paths: Vec<PathBuf>,
    pub header_rules: Vec<HeaderRule>,
    pub default_header_rules: Vec<HeaderRuleId>,

    /// Additional configuration for our subgraphs
    pub subgraph_configs: BTreeMap<SubgraphId, SubgraphConfig>,

    /// Caching configuration
    #[serde(default)]
    pub cache: CacheConfigs,

    pub auth: Option<AuthConfig>,

    #[serde(default)]
    pub operation_limits: OperationLimits,

    #[serde(default)]
    pub disable_introspection: bool,

    #[serde(default)]
    pub rate_limit: Option<RateLimitConfig>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<Duration>,

    #[serde(default)]
    pub entity_caching: EntityCaching,

    #[serde(default)]
    pub retry: Option<RetryConfig>,

    #[serde(default)]
    pub batching: BatchingConfig,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct BatchingConfig {
    pub enabled: bool,
    pub limit: Option<usize>,
}

impl Config {
    pub fn from_graph(graph: FederatedGraph) -> Self {
        Config {
            graph,
            strings: Vec::new(),
            paths: Vec::new(),
            header_rules: Vec::new(),
            default_header_rules: Default::default(),
            subgraph_configs: Default::default(),
            cache: Default::default(),
            auth: Default::default(),
            operation_limits: Default::default(),
            disable_introspection: Default::default(),
            rate_limit: Default::default(),
            timeout: None,
            entity_caching: EntityCaching::Disabled,
            retry: None,
            batching: Default::default(),
        }
    }

    pub fn rate_limit_config(&self) -> Option<RateLimitConfigRef<'_>> {
        self.rate_limit.map(|config| RateLimitConfigRef {
            storage: config.storage,
            redis: RateLimitRedisConfigRef {
                url: &self[config.redis.url],
                key_prefix: &self[config.redis.key_prefix],
                tls: config.redis.tls.map(|config| RateLimitRedisTlsConfigRef {
                    cert: config.cert.map(|cert| &self[cert]),
                    key: config.key.map(|key| &self[key]),
                    ca: config.ca.map(|ca| &self[ca]),
                }),
            },
        })
    }

    pub fn as_keyed_rate_limit_config(&self) -> Vec<(RateLimitKey<'_>, GraphRateLimit)> {
        let mut key_based_config = Vec::new();

        if let Some(global_config) = self.rate_limit.as_ref().and_then(|c| c.global) {
            key_based_config.push((RateLimitKey::Global, global_config));
        }

        for subgraph in self.subgraph_configs.values() {
            if let Some(subgraph_rate_limit) = subgraph.rate_limit {
                let key = RateLimitKey::Subgraph(&self.strings[subgraph.name.0]);
                key_based_config.push((key, subgraph_rate_limit));
            }
        }

        key_based_config
    }
}

impl std::ops::Index<StringId> for Config {
    type Output = String;

    fn index(&self, index: StringId) -> &String {
        &self.strings[index.0]
    }
}

impl std::ops::Index<PathId> for Config {
    type Output = Path;

    fn index(&self, index: PathId) -> &Path {
        &self.paths[index.0]
    }
}

pub(crate) fn serialize_to_federated_sdl<S>(graph: &FederatedGraph, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let rendered = federated_graph::render_federated_sdl(graph)
        .map_err(|err| serde::ser::Error::custom(format!("Failed to render federated SDL: {err}",)))?;

    serializer.serialize_str(&rendered)
}

pub(crate) fn deserialize_from_federated_sdl<'de, D>(deserializer: D) -> Result<FederatedGraph, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct FederatedSdlVisitor;

    impl<'a> serde::de::Visitor<'a> for FederatedSdlVisitor {
        type Value = FederatedGraph;

        fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
            federated_graph::from_sdl(v).map_err(E::custom)
        }

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a federated SDL string")
        }
    }

    deserializer.deserialize_str(FederatedSdlVisitor)
}
