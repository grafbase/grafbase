//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
mod graphql;

use crate::prelude::*;
pub use graphql::*;
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union Subgraph @id @meta(module: "subgraph") @variants(empty: ["Introspection"]) = GraphqlEndpoint
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubgraphId {
    GraphqlEndpoint(GraphqlEndpointId),
    Introspection,
}

impl std::fmt::Debug for SubgraphId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubgraphId::GraphqlEndpoint(variant) => variant.fmt(f),
            SubgraphId::Introspection => write!(f, "Introspection"),
        }
    }
}

impl From<GraphqlEndpointId> for SubgraphId {
    fn from(value: GraphqlEndpointId) -> Self {
        SubgraphId::GraphqlEndpoint(value)
    }
}

#[derive(Clone, Copy)]
pub enum Subgraph<'a> {
    GraphqlEndpoint(GraphqlEndpoint<'a>),
    Introspection,
}

impl std::fmt::Debug for Subgraph<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subgraph::GraphqlEndpoint(variant) => variant.fmt(f),
            Subgraph::Introspection => write!(f, "Introspection"),
        }
    }
}

impl Walk<Schema> for SubgraphId {
    type Walker<'a> = Subgraph<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        match self {
            SubgraphId::GraphqlEndpoint(id) => Subgraph::GraphqlEndpoint(id.walk(schema)),
            SubgraphId::Introspection => Subgraph::Introspection,
        }
    }
}

impl Subgraph<'_> {
    pub fn id(&self) -> SubgraphId {
        match self {
            Subgraph::GraphqlEndpoint(walker) => SubgraphId::GraphqlEndpoint(walker.id),
            Subgraph::Introspection => SubgraphId::Introspection,
        }
    }
}
