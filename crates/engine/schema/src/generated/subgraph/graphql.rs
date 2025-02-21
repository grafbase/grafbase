//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    StringId, SubgraphConfig, SubscriptionProtocol, UrlId,
    generated::{ExtensionDirective, ExtensionDirectiveId, HeaderRule, HeaderRuleId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type GraphqlEndpoint @meta(module: "subgraph/graphql") @indexed(id_size: "u16") {
///   subgraph_name: String!
///   url: Url!
///   websocket_url: Url
///   header_rules: [HeaderRule!]!
///   config: SubgraphConfig!
///   "Schema directives applied by the given subgraph"
///   schema_directives: [ExtensionDirective!]! @vec
///   "The protocol to use for subscriptions from this subgraph"
///   subscription_protocol: SubscriptionProtocol!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GraphqlEndpointRecord {
    pub subgraph_name_id: StringId,
    pub url_id: UrlId,
    pub websocket_url_id: Option<UrlId>,
    pub header_rule_ids: IdRange<HeaderRuleId>,
    pub config: SubgraphConfig,
    /// Schema directives applied by the given subgraph
    pub schema_directive_ids: Vec<ExtensionDirectiveId>,
    /// The protocol to use for subscriptions from this subgraph
    pub subscription_protocol: SubscriptionProtocol,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct GraphqlEndpointId(std::num::NonZero<u16>);

#[derive(Clone, Copy)]
pub struct GraphqlEndpoint<'a> {
    pub(crate) schema: &'a Schema,
    pub id: GraphqlEndpointId,
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
    /// Schema directives applied by the given subgraph
    pub fn schema_directives(&self) -> impl Iter<Item = ExtensionDirective<'a>> + 'a {
        self.as_ref().schema_directive_ids.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for GraphqlEndpointId {
    type Walker<'w>
        = GraphqlEndpoint<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        GraphqlEndpoint {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for GraphqlEndpoint<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphqlEndpoint")
            .field("subgraph_name", &self.subgraph_name())
            .field("url", &self.url())
            .field("websocket_url", &self.websocket_url())
            .field("header_rules", &self.header_rules())
            .field("config", &self.config)
            .field("schema_directives", &self.schema_directives())
            .field("subscription_protocol", &self.subscription_protocol)
            .finish()
    }
}
