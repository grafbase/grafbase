//! Glue crate between parser-sdl & engine-v2-config

use std::collections::BTreeMap;

use engine_v2_config::{
    latest::{self as config},
    VersionedConfig,
};
use federated_graph::{FederatedGraph, FederatedGraphV1, SubgraphId};
use parser_sdl::federation::FederatedGraphConfig;

mod strings;

pub fn build_config(config: &FederatedGraphConfig, graph: FederatedGraph) -> VersionedConfig {
    let FederatedGraph::V1(graph) = graph;

    let mut strings = strings::Strings::default();
    let mut headers = vec![];
    let mut subgraph_configs = BTreeMap::new();

    for (name, config) in &config.subgraphs {
        let Some(subgraph_id) = graph.find_subgraph(name) else {
            continue;
        };

        let mut header_ids = Vec::with_capacity(config.headers.len());
        for (name, value) in &config.headers {
            let name = strings.intern(name);

            let value = match value {
                parser_sdl::federation::SubgraphHeaderValue::Static(value) => {
                    config::HeaderValue::Static(strings.intern(value))
                }
                parser_sdl::federation::SubgraphHeaderValue::Forward(value) => {
                    config::HeaderValue::Forward(strings.intern(value))
                }
            };

            header_ids.push(config::HeaderId(headers.len()));
            headers.push(config::Header { name, value })
        }

        subgraph_configs.insert(subgraph_id, config::SubgraphConfig { headers: header_ids });
    }

    VersionedConfig::V2(config::Config {
        graph,
        strings: strings.into_vec(),
        headers,
        subgraph_configs,
    })
}

pub trait FederatedGraphExt {
    fn find_subgraph(&self, name: &str) -> Option<SubgraphId>;
}

impl FederatedGraphExt for FederatedGraphV1 {
    fn find_subgraph(&self, name: &str) -> Option<SubgraphId> {
        self.subgraphs
            .iter()
            .enumerate()
            .find(|(_, subgraph)| self[subgraph.name] == name)
            .map(|(i, _)| SubgraphId(i))
    }
}
