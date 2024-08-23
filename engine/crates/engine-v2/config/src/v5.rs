mod header;
mod rate_limit;

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

use federated_graph::{FederatedGraphV3, SubgraphId};

pub(crate) use self::rate_limit::{RateLimitConfigRef, RateLimitRedisConfigRef, RateLimitRedisTlsConfigRef};

pub use super::v2::EntityCaching;
pub use super::v4::{
    AuthConfig, AuthProviderConfig, CacheConfig, CacheConfigTarget, CacheConfigs, Header, HeaderId, HeaderValue,
    JwksConfig, JwtConfig, OperationLimits, RetryConfig, StringId, SubgraphConfig,
};
pub use header::{
    HeaderForward, HeaderInsert, HeaderRemove, HeaderRenameDuplicate, HeaderRule, HeaderRuleId, NameOrPattern,
};
pub use rate_limit::{
    GraphRateLimit, RateLimitConfig, RateLimitRedisConfig, RateLimitRedisTlsConfig, RateLimitStorage,
};

/// Configuration for a federated graph
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Config {
    pub graph: FederatedGraphV3,
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
}

impl Config {
    pub fn from_graph(graph: FederatedGraphV3) -> Self {
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

#[derive(Clone, Copy, Debug)]
pub enum RateLimitKey<'a> {
    Global,
    Subgraph(&'a str),
}

impl std::ops::Index<StringId> for Config {
    type Output = String;

    fn index(&self, index: StringId) -> &String {
        &self.strings[index.0]
    }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize, Debug)]
pub struct PathId(pub usize);

impl std::ops::Index<PathId> for Config {
    type Output = Path;

    fn index(&self, index: PathId) -> &Path {
        &self.paths[index.0]
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, time::Duration};

    use federated_graph::{FederatedGraphV3, FieldId, ObjectId, RootOperationTypes};

    use crate::v5::{CacheConfig, CacheConfigTarget, CacheConfigs, Config};

    #[test]
    fn make_sure_we_can_serialize_the_config() {
        let mut cache_config = BTreeMap::<CacheConfigTarget, CacheConfig>::new();
        cache_config.insert(
            CacheConfigTarget::Field(FieldId(0)),
            CacheConfig {
                max_age: Duration::from_secs(0),
                stale_while_revalidate: Duration::from_secs(0),
            },
        );

        let config = Config {
            graph: FederatedGraphV3 {
                subgraphs: vec![],
                root_operation_types: RootOperationTypes {
                    query: ObjectId(0),
                    mutation: None,
                    subscription: None,
                },
                objects: vec![],
                interfaces: vec![],
                fields: vec![],
                enums: vec![],
                unions: vec![],
                scalars: vec![],
                input_objects: vec![],
                strings: vec![],
                input_value_definitions: vec![],
                enum_values: vec![],
                directives: vec![],
                authorized_directives: vec![],
                field_authorized_directives: vec![],
                object_authorized_directives: vec![],
                interface_authorized_directives: vec![],
            },
            strings: vec![],
            paths: Vec::new(),
            header_rules: vec![],
            default_header_rules: Vec::new(),
            subgraph_configs: Default::default(),
            cache: CacheConfigs { rules: cache_config },
            auth: None,
            operation_limits: Default::default(),
            disable_introspection: Default::default(),
            rate_limit: Default::default(),
            timeout: None,
            entity_caching: Default::default(),
            retry: None,
        };

        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!(serde_json::json!(config), @r###"
            {
              "auth": null,
              "cache": {
                "rules": {
                  "f0": {
                    "max_age": {
                      "nanos": 0,
                      "secs": 0
                    },
                    "stale_while_revalidate": {
                      "nanos": 0,
                      "secs": 0
                    }
                  }
                }
              },
              "default_header_rules": [],
              "disable_introspection": false,
              "entity_caching": "Disabled",
              "graph": {
                "authorized_directives": [],
                "directives": [],
                "enum_values": [],
                "enums": [],
                "field_authorized_directives": [],
                "fields": [],
                "input_objects": [],
                "input_value_definitions": [],
                "interface_authorized_directives": [],
                "interfaces": [],
                "object_authorized_directives": [],
                "objects": [],
                "root_operation_types": {
                  "mutation": null,
                  "query": 0,
                  "subscription": null
                },
                "scalars": [],
                "strings": [],
                "subgraphs": [],
                "unions": []
              },
              "header_rules": [],
              "operation_limits": {
                "aliases": null,
                "complexity": null,
                "depth": null,
                "height": null,
                "rootFields": null
              },
              "paths": [],
              "rate_limit": null,
              "retry": null,
              "strings": [],
              "subgraph_configs": {}
            }
            "###);
        });
    }
}
