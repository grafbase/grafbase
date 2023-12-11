use crate::{FieldSet, Header, HeaderId, HeaderValue, SchemaWalker, StringId, SubgraphId};

#[derive(Default)]
pub struct DataSource {
    pub(crate) subgraphs: Vec<Subgraph>,
}

#[derive(Debug)]
pub struct Subgraph {
    pub name: StringId,
    pub url: StringId,
    pub headers: Vec<HeaderId>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RootFieldResolver {
    pub subgraph_id: SubgraphId,
}

pub type RootFieldResolverWalker<'a> = SchemaWalker<'a, &'a RootFieldResolver>;

impl<'a> std::ops::Deref for RootFieldResolverWalker<'a> {
    type Target = RootFieldResolver;

    fn deref(&self) -> &'a Self::Target {
        self.wrapped
    }
}

impl<'a> RootFieldResolverWalker<'a> {
    pub fn name(&self) -> String {
        format!(
            "Federation root field resolver for subgraph '{}'",
            self.schema[self.data_source()[self.subgraph_id].name],
        )
    }

    pub fn data_source(&self) -> &'a DataSource {
        &self.schema.data_sources.federation
    }

    pub fn subgraph(&self) -> SubgraphWalker<'a> {
        self.walk(&self.data_source()[self.subgraph_id])
    }
}

impl<'a> std::fmt::Debug for RootFieldResolverWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FederationRootField")
            .field("subgraph", &self.subgraph().name())
            .finish()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct EntityResolver {
    pub subgraph_id: SubgraphId,
    pub key: Key,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Key {
    pub fields: FieldSet,
}

pub type EntityResolverWalker<'a> = SchemaWalker<'a, &'a EntityResolver>;

impl<'a> std::ops::Deref for EntityResolverWalker<'a> {
    type Target = EntityResolver;

    fn deref(&self) -> &'a Self::Target {
        self.wrapped
    }
}

impl<'a> EntityResolverWalker<'a> {
    pub fn name(&self) -> String {
        format!(
            "Federation entity resolver for subgraph '{}'",
            self.schema[self.data_source()[self.subgraph_id].name],
        )
    }

    pub fn data_source(&self) -> &'a DataSource {
        &self.schema.data_sources.federation
    }

    pub fn subgraph(&self) -> SubgraphWalker<'a> {
        self.walk(&self.data_source()[self.subgraph_id])
    }
}

impl<'a> std::fmt::Debug for EntityResolverWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FederationEntityResolver")
            .field("subgraph", &self.subgraph().name())
            .field("key", &self.walk(&self.key.fields))
            .finish()
    }
}

pub type SubgraphWalker<'a> = SchemaWalker<'a, &'a Subgraph>;

impl<'a> SubgraphWalker<'a> {
    pub fn name(&self) -> &'a str {
        &self.schema[self.wrapped.name]
    }

    pub fn url(&self) -> &'a str {
        &self.schema[self.wrapped.url]
    }

    pub fn headers(&self) -> impl Iterator<Item = SubgraphHeaderWalker<'a>> + '_ {
        self.schema
            .default_headers
            .iter()
            .chain(self.wrapped.headers.iter())
            .map(|id| self.walk(&self.schema[*id]))
    }
}

impl<'a> std::fmt::Debug for SubgraphWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subgraph")
            .field("name", &self.name())
            .field("url", &self.url())
            .finish()
    }
}

pub type SubgraphHeaderWalker<'a> = SchemaWalker<'a, &'a Header>;

impl<'a> SubgraphHeaderWalker<'a> {
    pub fn name(&self) -> &'a str {
        &self.schema[self.wrapped.name]
    }

    pub fn value(&self) -> SubgraphHeaderValueRef<'a> {
        match self.wrapped.value {
            HeaderValue::Forward(id) => SubgraphHeaderValueRef::Forward(&self.schema[id]),
            HeaderValue::Static(id) => SubgraphHeaderValueRef::Static(&self.schema[id]),
        }
    }
}

#[derive(Debug)]
pub enum SubgraphHeaderValueRef<'a> {
    Forward(&'a str),
    Static(&'a str),
}

impl<'a> std::fmt::Debug for SubgraphHeaderWalker<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubgraphHeaderWalker")
            .field("name", &self.name())
            .field("value", &self.value())
            .finish()
    }
}
