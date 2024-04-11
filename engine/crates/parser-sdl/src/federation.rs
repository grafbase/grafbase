use std::collections::BTreeMap;

use crate::{rules::auth_directive::v2::AuthV2Directive, GlobalCacheRules};
use engine::registry::{ConnectorHeaderValue, OperationLimits};

/// Configuration for a federated graph
#[derive(Clone, Debug, Default)]
pub struct FederatedGraphConfig {
    pub subgraphs: BTreeMap<String, SubgraphConfig>,

    pub default_headers: Vec<(String, SubgraphHeaderValue)>,

    pub operation_limits: OperationLimits,

    pub global_cache_rules: GlobalCacheRules<'static>,

    pub auth: Option<AuthV2Directive>,

    pub disable_introspection: bool,
}

/// Configuration for a subgraph of the current federated graph
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubgraphConfig {
    /// The name of the subgrah
    pub name: String,

    /// The URL to use in development
    ///
    /// This is only used in development and should be ignored in deployed
    /// environments
    pub development_url: Option<String>,

    /// The URL to use for GraphQL-WS calls.
    ///
    /// This will default to the normal URL if not present.
    pub websocket_url: Option<String>,

    /// Any headers we should forward for this subgraph
    pub headers: Vec<(String, SubgraphHeaderValue)>,
}

/// The value of a header to send to a subgraph
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
