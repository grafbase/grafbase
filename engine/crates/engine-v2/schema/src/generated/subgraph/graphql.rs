//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{HeaderRule, HeaderRuleId},
    prelude::*,
    StringId, SubgraphConfig, UrlId,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type GraphqlEndpoint @meta(module: "subgraph/graphql") @indexed(id_size: "u32", max_id: "MAX_ID") {
///   subgraph_name: String!
///   url: Url!
///   websocket_url: Url
///   header_rules: [HeaderRule!]!
///   config: SubgraphConfig!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GraphqlEndpointRecord {
    pub subgraph_name_id: StringId,
    pub url_id: UrlId,
    pub websocket_url_id: Option<UrlId>,
    pub header_rule_ids: Vec<HeaderRuleId>,
    pub config: SubgraphConfig,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct GraphqlEndpointId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct GraphqlEndpoint<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) id: GraphqlEndpointId,
}

impl std::ops::Deref for GraphqlEndpoint<'_> {
    type Target = GraphqlEndpointRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> GraphqlEndpoint<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a GraphqlEndpointRecord {
        &self.schema[self.id]
    }
    pub fn id(&self) -> GraphqlEndpointId {
        self.id
    }
    pub fn subgraph_name(&self) -> &'a str {
        self.subgraph_name_id.walk(self.schema)
    }
    pub fn url(&self) -> &'a Url {
        self.url_id.walk(self.schema)
    }
    pub fn websocket_url(&self) -> Option<&'a Url> {
        self.websocket_url_id.walk(self.schema)
    }
    pub fn header_rules(&self) -> impl Iter<Item = HeaderRule<'a>> + 'a {
        self.as_ref().header_rule_ids.walk(self.schema)
    }
}

impl Walk<Schema> for GraphqlEndpointId {
    type Walker<'a> = GraphqlEndpoint<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        GraphqlEndpoint { schema, id: self }
    }
}

impl std::fmt::Debug for GraphqlEndpoint<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphqlEndpoint")
            .field("subgraph_name", &self.subgraph_name())
            .field("url", &self.url())
            .field("websocket_url", &self.websocket_url())
            .field(
                "header_rules",
                &self.header_rules().map(|walker| walker.to_string()).collect::<Vec<_>>(),
            )
            .field("config", &self.config)
            .finish()
    }
}
