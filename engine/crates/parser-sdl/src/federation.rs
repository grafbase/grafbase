use std::collections::BTreeMap;

use crate::GlobalCacheRules;
use engine::registry::ConnectorHeaderValue;

/// Configuration for a federated graph
#[derive(Clone, Debug, Default)]
pub struct FederatedGraphConfig {
    pub subgraphs: BTreeMap<String, SubgraphConfig>,

    pub default_headers: Vec<(String, SubgraphHeaderValue)>,

    pub global_cache_rules: GlobalCacheRules<'static>,
}

/// Configuration for a subgraph of the current federated graph
#[derive(Clone, Debug, Default)]
pub struct SubgraphConfig {
    /// The name of the subgrah
    pub name: String,

    /// Any headers we should forward for this subgraph
    pub headers: Vec<(String, SubgraphHeaderValue)>,
}

/// The value of a header to send to a subgraph
#[derive(Clone, Debug)]
pub enum SubgraphHeaderValue {
    /// We should send a static value for this header
    Static(String),
    /// We should pull the value for this header from the named header in the incoming
    /// request
    Forward(String),
}

impl From<ConnectorHeaderValue> for SubgraphHeaderValue {
    fn from(value: ConnectorHeaderValue) -> Self {
        match value {
            ConnectorHeaderValue::Static(value) => SubgraphHeaderValue::Static(value),
            ConnectorHeaderValue::Forward(value) => SubgraphHeaderValue::Forward(value),
        }
    }
}
