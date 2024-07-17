use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use regex::Regex;

use crate::GLOBAL_RATE_LIMIT_KEY;
use federated_graph::{FederatedGraphV3, SubgraphId};

pub use super::v4::{
    AuthConfig, AuthProviderConfig, CacheConfig, CacheConfigTarget, CacheConfigs, Header, HeaderId, HeaderValue,
    JwksConfig, JwtConfig, OperationLimits, StringId, SubgraphConfig,
};

/// Configuration for a federated graph
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Config {
    pub graph: FederatedGraphV3,
    pub strings: Vec<String>,
    pub header_rules: Vec<HeaderRule>,

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
}

impl Config {
    pub fn from_graph(graph: FederatedGraphV3) -> Self {
        Config {
            graph,
            strings: Vec::new(),
            header_rules: Vec::new(),
            subgraph_configs: Default::default(),
            cache: Default::default(),
            auth: Default::default(),
            operation_limits: Default::default(),
            disable_introspection: Default::default(),
            rate_limit: Default::default(),
        }
    }

    pub fn as_keyed_rate_limit_config(&self) -> HashMap<&str, RateLimitConfig> {
        let mut key_based_config = HashMap::new();
        if let Some(global_config) = &self.rate_limit {
            key_based_config.insert(GLOBAL_RATE_LIMIT_KEY, global_config.clone());
        }

        for subgraph in self.subgraph_configs.values() {
            if let Some(subgraph_rate_limit) = &subgraph.rate_limit {
                key_based_config.insert(&self.strings[subgraph.name.0], subgraph_rate_limit.clone());
            }
        }

        key_based_config
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RateLimitConfig {
    pub limit: usize,
    pub duration: Duration,
}

/// A header name can be provided either as a regex or as a static name.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub enum NameOrPattern {
    /// A regex pattern matching multiple headers.
    #[serde(with = "serde_regex", rename = "pattern")]
    Pattern(Regex),
    /// A static single name.
    #[serde(rename = "name")]
    Name(StringId),
}

impl From<StringId> for NameOrPattern {
    fn from(value: StringId) -> Self {
        Self::Name(value)
    }
}

/// Defines a header rule, executed in order before anything else in the engine.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(tag = "rule")]
pub enum HeaderRule {
    /// Forward the header to the subgraphs.
    #[serde(rename = "forward")]
    Forward(HeaderForward),
    /// Insert a new static header.
    #[serde(rename = "insert")]
    Insert(HeaderInsert),
    /// Remove the header.
    #[serde(rename = "remove")]
    Remove(HeaderRemove),
}

/// Header forwarding rules.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct HeaderForward {
    /// Name or pattern of the header to be forwarded.
    #[serde(flatten)]
    pub name: NameOrPattern,
    /// If header is not present, insert this value.
    pub default: Option<StringId>,
    /// Use this name instead of the original when forwarding.
    pub rename: Option<StringId>,
}

/// Header insertion rules.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct HeaderInsert {
    /// The name of the header.
    pub name: StringId,
    /// The value of the header.
    pub value: StringId,
}

/// Header removal rules
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct HeaderRemove {
    /// Removes the header with a static name or matching a regex pattern.
    #[serde(flatten)]
    pub name: NameOrPattern,
}

impl std::ops::Index<StringId> for Config {
    type Output = String;

    fn index(&self, index: StringId) -> &String {
        &self.strings[index.0]
    }
}

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct HeaderRuleId(pub usize);

impl From<HeaderId> for HeaderRuleId {
    fn from(value: HeaderId) -> Self {
        Self(value.0)
    }
}

impl std::ops::Index<HeaderRuleId> for Config {
    type Output = HeaderRule;

    fn index(&self, index: HeaderRuleId) -> &Self::Output {
        &self.header_rules[index.0]
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
            header_rules: vec![],
            subgraph_configs: Default::default(),
            cache: CacheConfigs { rules: cache_config },
            auth: None,
            operation_limits: Default::default(),
            disable_introspection: Default::default(),
            rate_limit: Default::default(),
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
              "disable_introspection": false,
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
              "rate_limit": null,
              "strings": [],
              "subgraph_configs": {}
            }
            "###);
        });
    }
}
