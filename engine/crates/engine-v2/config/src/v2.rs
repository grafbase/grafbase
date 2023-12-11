use std::collections::BTreeMap;

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

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
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
