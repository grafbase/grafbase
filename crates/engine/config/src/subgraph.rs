use std::time::Duration;

use federated_graph::StringId;
use gateway_config::SubscriptionProtocol;
use url::Url;

use crate::{EntityCaching, GraphRateLimit, HeaderRuleId, RetryConfig};

/// Additional configuration for a particular subgraph
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct SubgraphConfig {
    pub name: StringId,
    pub url: Option<Url>,
    pub websocket_url: Option<StringId>,
    pub subscription_protocol: Option<SubscriptionProtocol>,
    pub headers: Vec<HeaderRuleId>,
    #[serde(default)]
    pub rate_limit: Option<GraphRateLimit>,
    #[serde(default)]
    pub timeout: Option<Duration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryConfig>,
    #[serde(default)]
    pub entity_caching: Option<EntityCaching>,
}
