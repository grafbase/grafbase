use std::collections::BTreeMap;

pub use super::v2::{
    AuthConfig, AuthProviderConfig, CacheConfig, CacheConfigTarget, CacheConfigs, Header, HeaderId, HeaderValue,
    JwksConfig, JwtConfig, OperationLimits, StringId, SubgraphConfig,
};

use federated_graph::{FederatedGraphV2, SubgraphId};

/// Configuration for a federated graph
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub graph: FederatedGraphV2,
    pub strings: Vec<String>,
    pub headers: Vec<Header>,

    /// Default headers that should be sent to every subgraph
    pub default_headers: Vec<HeaderId>,

    /// Additional configuration for our subgraphs
    pub subgraph_configs: BTreeMap<SubgraphId, SubgraphConfig>,

    /// Caching configuration
    #[serde(default)]
    pub cache: CacheConfigs,

    pub auth: Option<AuthConfig>,

    #[serde(default)]
    pub operation_limits: OperationLimits,
}

impl std::ops::Index<StringId> for Config {
    type Output = String;

    fn index(&self, index: StringId) -> &String {
        &self.strings[index.0]
    }
}

impl std::ops::Index<HeaderId> for Config {
    type Output = Header;

    fn index(&self, index: HeaderId) -> &Header {
        &self.headers[index.0]
    }
}

#[cfg(test)]
mod tests {
    use crate::v3::{CacheConfig, CacheConfigTarget, CacheConfigs, Config};
    use federated_graph::{FederatedGraphV2, FieldId, ObjectId, RootOperationTypes};
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
            graph: FederatedGraphV2 {
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
                input_value_definitions: vec![],
                enum_values: vec![],
                directives: vec![],
            },
            strings: vec![],
            headers: vec![],
            default_headers: vec![],
            subgraph_configs: Default::default(),
            cache: CacheConfigs { rules: cache_config },
            auth: None,
            operation_limits: Default::default(),
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
              "operation_limits": {
                "aliases": null,
                "complexity": null,
                "depth": null,
                "height": null,
                "rootFields": null
              },
              "strings": [],
              "subgraph_configs": {}
            }
            "###);
        });
    }
}
