use std::time::Duration;

use walker::Walk;

use crate::{ExtensionDirective, ExtensionDirectiveId, RetryConfig, Subgraph};

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

    /// Defines whether resolvers should also be treated as boundaries in addition to being
    /// entrypoints. Meaning that if we encounter a field with a resolver from the same subgraph,
    /// do we need to call it or can we just provide this field from the parent resolver?
    pub fn resolvers_define_boundaries(&self) -> bool {
        match self {
            Subgraph::GraphqlEndpoint(_) => false,
            Subgraph::Virtual(_) => true,
            Subgraph::Introspection(_) => false,
        }
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
