//! Glue crate between parser-sdl & engine-v2-config

use std::collections::BTreeMap;

use engine_v2_config::{
    latest::{self as config, Header, HeaderId},
    VersionedConfig,
};
use federated_graph::{FederatedGraph, FederatedGraphV1, SubgraphId};
use parser_sdl::federation::{FederatedGraphConfig, SubgraphHeaderValue};

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

    VersionedConfig::V2(config::Config {
        graph,
        default_headers,
        strings: context.strings.into_vec(),
        headers: context.headers,
        subgraph_configs,
    })
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
