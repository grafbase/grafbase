//! Glue crate between parser-sdl & engine-v2-config

use std::collections::BTreeMap;

use engine_v2_config::latest::{CacheConfig, CacheConfigTarget};
use engine_v2_config::{
    latest::{self as config, Header, HeaderId},
    VersionedConfig,
};
use federated_graph::{FederatedGraph, FederatedGraphV1, FieldId, ObjectId, SubgraphId};
use parser_sdl::federation::{FederatedGraphConfig, SubgraphHeaderValue};
use parser_sdl::GlobalCacheTarget;

mod strings;

pub fn build_config(config: &FederatedGraphConfig, graph: FederatedGraph) -> VersionedConfig {
    let FederatedGraph::V1(graph) = graph;

    let mut context = BuildContext::default();
    let mut subgraph_configs = BTreeMap::new();

    let default_headers = context.insert_headers(&config.default_headers);

    for (name, config) in &config.subgraphs {
        let Some(subgraph_id) = graph.find_subgraph(name) else {
            continue;
        };

        let headers = context.insert_headers(&config.headers);

        subgraph_configs.insert(subgraph_id, config::SubgraphConfig { headers });
    }

    let cache_config = build_cache_config(config, &graph);

    VersionedConfig::V2(config::Config {
        graph,
        default_headers,
        strings: context.strings.into_vec(),
        headers: context.headers,
        subgraph_configs,
        cache_config,
    })
}

fn build_cache_config(
    config: &FederatedGraphConfig,
    graph: &FederatedGraphV1,
) -> BTreeMap<CacheConfigTarget, CacheConfig> {
    let mut cache_config = BTreeMap::new();

    for (target, cache_control) in config.global_cache_rules.iter() {
        match target {
            GlobalCacheTarget::Type(name) => {
                if let Some(object_id) = graph.find_object(name) {
                    cache_config.insert(
                        CacheConfigTarget::Object(object_id),
                        CacheConfig {
                            public: cache_control.public,
                            max_age: cache_control.max_age,
                            stale_while_revalidate: cache_control.stale_while_revalidate,
                        },
                    );
                }
            }
            GlobalCacheTarget::Field(object_name, field_name) => {
                if let Some(field_id) = graph.find_object_field(object_name, field_name) {
                    cache_config.insert(
                        CacheConfigTarget::Field(field_id),
                        CacheConfig {
                            public: cache_control.public,
                            max_age: cache_control.max_age,
                            stale_while_revalidate: cache_control.stale_while_revalidate,
                        },
                    );
                }
            }
        }
    }

    cache_config
}

#[derive(Default)]
struct BuildContext<'a> {
    strings: strings::Strings<'a>,
    headers: Vec<Header>,
}

impl<'a> BuildContext<'a> {
    pub fn insert_headers(
        &mut self,
        headers: impl IntoIterator<Item = &'a (String, SubgraphHeaderValue)>,
    ) -> Vec<HeaderId> {
        headers
            .into_iter()
            .map(|(name, value)| self.insert_header(name, value))
            .collect()
    }

    pub fn insert_header(&mut self, name: &'a str, value: &'a SubgraphHeaderValue) -> HeaderId {
        let name = self.strings.intern(name);

        let value = match value {
            parser_sdl::federation::SubgraphHeaderValue::Static(value) => {
                config::HeaderValue::Static(self.strings.intern(value))
            }
            parser_sdl::federation::SubgraphHeaderValue::Forward(value) => {
                config::HeaderValue::Forward(self.strings.intern(value))
            }
        };

        let id = config::HeaderId(self.headers.len());
        self.headers.push(config::Header { name, value });
        id
    }
}

pub trait FederatedGraphExt {
    fn find_subgraph(&self, name: &str) -> Option<SubgraphId>;
    fn find_object(&self, name: &str) -> Option<ObjectId>;
    fn find_object_field(&self, object_name: &str, field_name: &str) -> Option<FieldId>;
}

impl FederatedGraphExt for FederatedGraphV1 {
    fn find_subgraph(&self, name: &str) -> Option<SubgraphId> {
        self.subgraphs
            .iter()
            .enumerate()
            .find(|(_, subgraph)| self[subgraph.name] == name)
            .map(|(i, _)| SubgraphId(i))
    }

    fn find_object(&self, name: &str) -> Option<ObjectId> {
        self.objects
            .iter()
            .enumerate()
            .find(|(_, object)| self[object.name] == name)
            .map(|(i, _)| ObjectId(i))
    }

    fn find_object_field(&self, object_name: &str, field_name: &str) -> Option<FieldId> {
        self.object_fields
            .iter()
            .enumerate()
            .find(|(_, object_field)| {
                let object = &self[object_field.object_id];
                let field = &self[object_field.field_id];

                self[object.name] == object_name && self[field.name] == field_name
            })
            .map(|(_, object_field)| object_field.field_id)
    }
}
