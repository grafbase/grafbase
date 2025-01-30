//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
mod graphql;
mod virtual_;

use crate::prelude::*;
pub use graphql::*;
pub use virtual_::*;
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union Subgraph @id @meta(module: "subgraph") @variants(empty: ["Introspection"], remove_suffix: "Subgraph") =
///   | GraphqlEndpoint
///   | VirtualSubgraph
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubgraphId {
    GraphqlEndpoint(GraphqlEndpointId),
    Introspection,
    Virtual(VirtualSubgraphId),
}

impl std::fmt::Debug for SubgraphId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubgraphId::GraphqlEndpoint(variant) => variant.fmt(f),
            SubgraphId::Introspection => write!(f, "Introspection"),
            SubgraphId::Virtual(variant) => variant.fmt(f),
        }
    }
}

impl From<GraphqlEndpointId> for SubgraphId {
    fn from(value: GraphqlEndpointId) -> Self {
        SubgraphId::GraphqlEndpoint(value)
    }
}
impl From<VirtualSubgraphId> for SubgraphId {
    fn from(value: VirtualSubgraphId) -> Self {
        SubgraphId::Virtual(value)
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
    pub fn is_virtual(&self) -> bool {
        matches!(self, SubgraphId::Virtual(_))
    }
    pub fn as_virtual(&self) -> Option<VirtualSubgraphId> {
        match self {
            SubgraphId::Virtual(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Subgraph<'a> {
    GraphqlEndpoint(GraphqlEndpoint<'a>),
    Introspection(&'a Schema),
    Virtual(VirtualSubgraph<'a>),
}

impl std::fmt::Debug for Subgraph<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subgraph::GraphqlEndpoint(variant) => variant.fmt(f),
            Subgraph::Introspection(_) => write!(f, "Introspection"),
            Subgraph::Virtual(variant) => variant.fmt(f),
        }
    }
}

impl<'a> From<GraphqlEndpoint<'a>> for Subgraph<'a> {
    fn from(item: GraphqlEndpoint<'a>) -> Self {
        Subgraph::GraphqlEndpoint(item)
    }
}
impl<'a> From<VirtualSubgraph<'a>> for Subgraph<'a> {
    fn from(item: VirtualSubgraph<'a>) -> Self {
        Subgraph::Virtual(item)
    }
}

impl<'a> Walk<&'a Schema> for SubgraphId {
    type Walker<'w>
        = Subgraph<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let schema: &'a Schema = schema.into();
        match self {
            SubgraphId::GraphqlEndpoint(id) => Subgraph::GraphqlEndpoint(id.walk(schema)),
            SubgraphId::Introspection => Subgraph::Introspection(schema),
            SubgraphId::Virtual(id) => Subgraph::Virtual(id.walk(schema)),
        }
    }
}

impl<'a> Subgraph<'a> {
    pub fn id(&self) -> SubgraphId {
        match self {
            Subgraph::GraphqlEndpoint(walker) => SubgraphId::GraphqlEndpoint(walker.id),
            Subgraph::Introspection(_) => SubgraphId::Introspection,
            Subgraph::Virtual(walker) => SubgraphId::Virtual(walker.id),
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
    pub fn is_virtual(&self) -> bool {
        matches!(self, Subgraph::Virtual(_))
    }
    pub fn as_virtual(&self) -> Option<VirtualSubgraph<'a>> {
        match self {
            Subgraph::Virtual(item) => Some(*item),
            _ => None,
        }
    }
}
