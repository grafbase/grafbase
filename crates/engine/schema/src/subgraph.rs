use std::time::Duration;

use walker::{Iter, Walk};

use crate::{ExtensionDirective, ExtensionDirectiveId, HeaderRule, RetryConfig, Subgraph};

impl<'a> Subgraph<'a> {
    pub fn name(&self) -> &'a str {
        match self {
            Subgraph::GraphqlEndpoint(endpoint) => endpoint.subgraph_name(),
            Subgraph::Virtual(subgraph) => subgraph.subgraph_name(),
            Subgraph::Introspection(_) => "introspection",
        }
    }

    pub fn extension_schema_directives(&self) -> impl Iterator<Item = ExtensionDirective<'_>> + '_ {
        static EMPTY_DIRECTIVES: &[ExtensionDirectiveId] = &[];

        let (schema, ids) = match self {
            Subgraph::GraphqlEndpoint(endpoint) => (endpoint.schema, endpoint.as_ref().schema_directive_ids.as_slice()),
            Subgraph::Introspection(schema) => (*schema, EMPTY_DIRECTIVES),
            Subgraph::Virtual(virt) => (virt.schema, virt.as_ref().schema_directive_ids.as_slice()),
        };

        ids.walk(schema)
    }

    pub fn header_rules(&self) -> impl Iter<Item = HeaderRule<'a>> + 'a {
        let (schema, ids) = match self {
            Subgraph::GraphqlEndpoint(endpoint) => (endpoint.schema, endpoint.header_rule_ids),
            Subgraph::Virtual(virtual_subgraph) => (virtual_subgraph.schema, virtual_subgraph.header_rule_ids),
            Subgraph::Introspection(_) => unreachable!(),
        };

        ids.walk(schema)
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
