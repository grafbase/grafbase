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
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union Subgraph @id @meta(module: "subgraph") @variants(empty: ["Introspection"], remove_suffix: "Subgraph") =
///   | GraphqlSubgraph
///   | VirtualSubgraph
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubgraphId {
    Graphql(GraphqlSubgraphId),
    Introspection,
    Virtual(VirtualSubgraphId),
}

impl std::fmt::Debug for SubgraphId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubgraphId::Graphql(variant) => variant.fmt(f),
            SubgraphId::Introspection => write!(f, "Introspection"),
            SubgraphId::Virtual(variant) => variant.fmt(f),
        }
    }
}

impl From<GraphqlSubgraphId> for SubgraphId {
    fn from(value: GraphqlSubgraphId) -> Self {
        SubgraphId::Graphql(value)
    }
}
impl From<VirtualSubgraphId> for SubgraphId {
    fn from(value: VirtualSubgraphId) -> Self {
        SubgraphId::Virtual(value)
    }
}

impl SubgraphId {
    pub fn is_graphql(&self) -> bool {
        matches!(self, SubgraphId::Graphql(_))
    }
    pub fn as_graphql(&self) -> Option<GraphqlSubgraphId> {
        match self {
            SubgraphId::Graphql(id) => Some(*id),
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
    Graphql(GraphqlSubgraph<'a>),
    Introspection(&'a Schema),
    Virtual(VirtualSubgraph<'a>),
}

impl std::fmt::Debug for Subgraph<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Subgraph::Graphql(variant) => variant.fmt(f),
            Subgraph::Introspection(_) => write!(f, "Introspection"),
            Subgraph::Virtual(variant) => variant.fmt(f),
        }
    }
}

impl<'a> From<GraphqlSubgraph<'a>> for Subgraph<'a> {
    fn from(item: GraphqlSubgraph<'a>) -> Self {
        Subgraph::Graphql(item)
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
            SubgraphId::Graphql(id) => Subgraph::Graphql(id.walk(schema)),
            SubgraphId::Introspection => Subgraph::Introspection(schema),
            SubgraphId::Virtual(id) => Subgraph::Virtual(id.walk(schema)),
        }
    }
}

impl<'a> Subgraph<'a> {
    pub fn id(&self) -> SubgraphId {
        match self {
            Subgraph::Graphql(walker) => SubgraphId::Graphql(walker.id),
            Subgraph::Introspection(_) => SubgraphId::Introspection,
            Subgraph::Virtual(walker) => SubgraphId::Virtual(walker.id),
        }
    }
    pub fn is_graphql(&self) -> bool {
        matches!(self, Subgraph::Graphql(_))
    }
    pub fn as_graphql(&self) -> Option<GraphqlSubgraph<'a>> {
        match self {
            Subgraph::Graphql(item) => Some(*item),
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
