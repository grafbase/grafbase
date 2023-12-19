use std::collections::BTreeMap;

pub use crate::cache_config::{CacheConfig, CacheConfigTarget};
use federated_graph::{FederatedGraphV1, SubgraphId};

/// Configuration for a federated graph
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub graph: FederatedGraphV1,
    pub strings: Vec<String>,
    pub headers: Vec<Header>,

    /// Default headers that should be sent to every subgraph
    pub default_headers: Vec<HeaderId>,

    /// Additional configuration for our subgraphs
    pub subgraph_configs: BTreeMap<SubgraphId, SubgraphConfig>,

    /// Caching configuration
    #[serde(default)]
    pub cache_config: BTreeMap<CacheConfigTarget, CacheConfig>,
}

/// Additional configuration for a particular subgraph
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SubgraphConfig {
    pub headers: Vec<HeaderId>,
}

/// A header that should be sent to a subgraph
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Header {
    pub name: StringId,
    pub value: HeaderValue,
}

/// The value that should be sent for a given header
#[derive(serde::Serialize, serde::Deserialize)]
pub enum HeaderValue {
    /// The given header from the current request should be forwarded
    /// to the subgraph
    Forward(StringId),
    /// The given string should always be sent
    Static(StringId),
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct StringId(pub usize);

impl std::ops::Index<StringId> for Config {
    type Output = String;

    fn index(&self, index: StringId) -> &String {
        &self.strings[index.0]
    }
}

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct HeaderId(pub usize);

impl std::ops::Index<HeaderId> for Config {
    type Output = Header;

    fn index(&self, index: HeaderId) -> &Header {
        &self.headers[index.0]
    }
}

#[cfg(test)]
mod tests {
    use crate::v2::{CacheConfig, CacheConfigTarget, Config};
    use federated_graph::{FederatedGraphV1, FieldId, ObjectId, RootOperationTypes};
    use std::collections::BTreeMap;
    use std::time::Duration;

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
            graph: FederatedGraphV1 {
                subgraphs: vec![],
                root_operation_types: RootOperationTypes {
                    query: ObjectId(0),
                    mutation: None,
                    subscription: None,
                },
                objects: vec![],
                object_fields: vec![],
                interfaces: vec![],
                interface_fields: vec![],
                fields: vec![],
                enums: vec![],
                unions: vec![],
                scalars: vec![],
                input_objects: vec![],
                strings: vec![],
                field_types: vec![],
            },
            strings: vec![],
            headers: vec![],
            default_headers: vec![],
            subgraph_configs: Default::default(),
            cache_config,
        };

        insta::assert_json_snapshot!(serde_json::json!(config), @r###"
        {
          "cache_config": {
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
          },
          "default_headers": [],
          "graph": {
            "enums": [],
            "field_types": [],
            "fields": [],
            "input_objects": [],
            "interface_fields": [],
            "interfaces": [],
            "object_fields": [],
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
          "headers": [],
          "strings": [],
          "subgraph_configs": {}
        }
        "###);
    }
}
