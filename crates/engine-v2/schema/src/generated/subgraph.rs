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

impl SubgraphId {
    pub fn is_graphql_endpoint(&self) -> bool {
        matches!(self, SubgraphId::GraphqlEndpoint(_))
    }
    pub fn as_graphql_endpoint(&self) -> Option<GraphqlEndpointId> {
        match self {
            SubgraphId::GraphqlEndpoint(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_introspection(&self) -> bool {
        matches!(self, SubgraphId::Introspection)
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

impl<'a> From<GraphqlEndpoint<'a>> for Subgraph<'a> {
    fn from(item: GraphqlEndpoint<'a>) -> Self {
        Subgraph::GraphqlEndpoint(item)
    }
}

impl<'a> Walk<&'a Schema> for SubgraphId {
    type Walker<'w> = Subgraph<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let schema: &'a Schema = schema.into();
        match self {
            SubgraphId::GraphqlEndpoint(id) => Subgraph::GraphqlEndpoint(id.walk(schema)),
            SubgraphId::Introspection => Subgraph::Introspection,
        }
    }
}

impl<'a> Subgraph<'a> {
    pub fn id(&self) -> SubgraphId {
        match self {
            Subgraph::GraphqlEndpoint(walker) => SubgraphId::GraphqlEndpoint(walker.id),
            Subgraph::Introspection => SubgraphId::Introspection,
        }
    }
    pub fn is_graphql_endpoint(&self) -> bool {
        matches!(self, Subgraph::GraphqlEndpoint(_))
    }
    pub fn as_graphql_endpoint(&self) -> Option<GraphqlEndpoint<'a>> {
        match self {
            Subgraph::GraphqlEndpoint(item) => Some(*item),
            _ => None,
        }
    }
}
