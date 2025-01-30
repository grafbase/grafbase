use std::time::Duration;

use walker::Walk;

use crate::{ExtensionDirective, Subgraph, TypeSystemDirectiveId};

impl<'a> Subgraph<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            Subgraph::GraphqlEndpoint(endpoint) => endpoint.subgraph_name(),
            Subgraph::Virtual(subgraph) => subgraph.subgraph_name(),
            Subgraph::Introspection(_) => "introspection",
        }
    }

    pub fn extension_schema_directives(&self) -> impl Iterator<Item = ExtensionDirective<'_>> + '_ {
        static EMPTY_DIRECTIVES: &[TypeSystemDirectiveId] = &[];

        let (schema, ids) = match self {
            Subgraph::GraphqlEndpoint(endpoint) => (endpoint.schema, endpoint.as_ref().schema_directive_ids.as_slice()),
            Subgraph::Introspection(schema) => (*schema, EMPTY_DIRECTIVES),
            Subgraph::Virtual(virt) => (virt.schema, virt.as_ref().schema_directive_ids.as_slice()),
        };
        ids.walk(schema).filter_map(|dir| dir.as_extension())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SubgraphConfig {
    pub timeout: Duration,
    pub retry: Option<RetryConfig>,
    // The ttl to use for caching for this subgraph.
    // If None then caching is disabled for this subgraph
    pub cache_ttl: Option<Duration>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct RetryConfig {
    /// How many retries are available per second, at a minimum.
    pub min_per_second: Option<u32>,
    /// Each successful request to the subgraph adds to the retry budget. This setting controls for how long the budget remembers successful requests.
    pub ttl: Option<Duration>,
    /// The fraction of the successful requests budget that can be used for retries.
    pub retry_percent: Option<f32>,
    /// Whether mutations should be retried at all. False by default.
    pub retry_mutations: bool,
}

impl From<config::RetryConfig> for RetryConfig {
    fn from(config: config::RetryConfig) -> Self {
        Self {
            min_per_second: config.min_per_second,
            ttl: config.ttl,
            retry_percent: config.retry_percent,
            retry_mutations: config.retry_mutations,
        }
    }
}
