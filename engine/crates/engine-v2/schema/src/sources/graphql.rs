use std::time::Duration;
use url::Url;

use crate::{
    HeaderRuleId, HeaderRuleWalker, RequiredFieldSet, RequiredFieldSetId, SchemaWalker, StringId, SubgraphId, UrlId,
};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct GraphqlEndpoints {
    pub(crate) endpoints: Vec<GraphqlEndpoint>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GraphqlEndpoint {
    pub(crate) subgraph_id: SubgraphId,
    pub(crate) name: StringId,
    pub(crate) url: UrlId,
    pub(crate) websocket_url: Option<UrlId>,
    pub(crate) header_rules: Vec<HeaderRuleId>,
    pub(crate) timeout: Duration,
}

id_newtypes::U8! {
    GraphqlEndpoints.endpoints[GraphqlEndpointId] => GraphqlEndpoint,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RootFieldResolver {
    pub(crate) endpoint_id: GraphqlEndpointId,
}

pub type RootFieldResolverWalker<'a> = SchemaWalker<'a, &'a RootFieldResolver>;

impl<'a> std::ops::Deref for RootFieldResolverWalker<'a> {
    type Target = RootFieldResolver;

    fn deref(&self) -> &'a Self::Target {
        self.item
    }
}

impl<'a> RootFieldResolverWalker<'a> {
    pub fn name(&self) -> String {
        format!("Graphql root field resolver for subgraph '{}'", self.endpoint().name())
    }

    pub fn subgraph_id(&self) -> SubgraphId {
        self.endpoint().as_ref().subgraph_id
    }

    pub fn endpoint(&self) -> GraphqlEndpointWalker<'a> {
        self.walk(self.endpoint_id)
    }
}

impl<'a> std::fmt::Debug for RootFieldResolverWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphqlRootField")
            .field("subgraph", &self.endpoint().name())
            .field("subgraph_id", &self.subgraph_id())
            .finish()
    }
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FederationEntityResolver {
    pub(crate) endpoint_id: GraphqlEndpointId,
    pub(crate) key: FederationKey,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FederationKey {
    pub(crate) fields: RequiredFieldSetId,
}

pub type FederationEntityResolverWalker<'a> = SchemaWalker<'a, &'a FederationEntityResolver>;

impl<'a> std::ops::Deref for FederationEntityResolverWalker<'a> {
    type Target = FederationEntityResolver;

    fn deref(&self) -> &'a Self::Target {
        self.item
    }
}

impl<'a> FederationEntityResolverWalker<'a> {
    pub fn name(&self) -> String {
        format!(
            "Graphql federation entity resolver for subgraph '{}'",
            self.endpoint().name()
        )
    }

    pub fn subgraph_id(&self) -> SubgraphId {
        self.endpoint().as_ref().subgraph_id
    }

    pub fn requires(&self) -> &'a RequiredFieldSet {
        &self.schema[self.key.fields]
    }

    pub fn endpoint(&self) -> GraphqlEndpointWalker<'a> {
        self.walk(self.endpoint_id)
    }
}

impl<'a> std::fmt::Debug for FederationEntityResolverWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphqlFederationEntityResolver")
            .field("subgraph", &self.endpoint().name())
            .field("subgraph_id", &self.subgraph_id())
            .field("key", &self.walk(&self.schema[self.key.fields]))
            .finish()
    }
}

pub type GraphqlEndpointWalker<'a> = SchemaWalker<'a, GraphqlEndpointId>;

impl<'a> GraphqlEndpointWalker<'a> {
    pub fn id(&self) -> GraphqlEndpointId {
        self.item
    }

    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a GraphqlEndpoint {
        &self.schema.data_sources.graphql[self.item]
    }

    pub fn name(&self) -> &'a str {
        &self.schema[self.as_ref().name]
    }

    pub fn timeout(self) -> Duration {
        self.as_ref().timeout
    }

    pub fn url(&self) -> &'a Url {
        &self.schema[self.as_ref().url]
    }

    pub fn websocket_url(&self) -> &'a Url {
        match self.as_ref().websocket_url {
            Some(websocket_id) => &self.schema[websocket_id],
            None => self.url(),
        }
    }

    pub fn header_rules(self) -> impl Iterator<Item = HeaderRuleWalker<'a>> {
        self.as_ref().header_rules.iter().map(move |id| self.walk(*id))
    }
}

impl<'a> std::fmt::Debug for GraphqlEndpointWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphqlEndpoint")
            .field("name", &self.name())
            .field("url", &self.url())
            .finish()
    }
}
