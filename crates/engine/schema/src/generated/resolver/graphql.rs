//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    FieldSet, FieldSetRecord,
    generated::{GraphqlSubgraph, GraphqlSubgraphId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type GraphqlRootFieldResolverDefinition @meta(module: "resolver/graphql") @copy {
///   endpoint: GraphqlSubgraph!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct GraphqlRootFieldResolverDefinitionRecord {
    pub subgraph_id: GraphqlSubgraphId,
}

#[derive(Clone, Copy)]
pub struct GraphqlRootFieldResolverDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: GraphqlRootFieldResolverDefinitionRecord,
}

impl std::ops::Deref for GraphqlRootFieldResolverDefinition<'_> {
    type Target = GraphqlRootFieldResolverDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> GraphqlRootFieldResolverDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &GraphqlRootFieldResolverDefinitionRecord {
        &self.item
    }
    pub fn subgraph(&self) -> GraphqlSubgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for GraphqlRootFieldResolverDefinitionRecord {
    type Walker<'w>
        = GraphqlRootFieldResolverDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        GraphqlRootFieldResolverDefinition {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for GraphqlRootFieldResolverDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphqlRootFieldResolverDefinition")
            .field("endpoint", &self.subgraph())
            .finish()
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type GraphqlFederationEntityResolverDefinition @meta(module: "resolver/graphql") {
///   endpoint: GraphqlSubgraph!
///   key_fields: FieldSet!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct GraphqlFederationEntityResolverDefinitionRecord {
    pub subgraph_id: GraphqlSubgraphId,
    pub key_fields_record: FieldSetRecord,
}

#[derive(Clone, Copy)]
pub struct GraphqlFederationEntityResolverDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a GraphqlFederationEntityResolverDefinitionRecord,
}

impl std::ops::Deref for GraphqlFederationEntityResolverDefinition<'_> {
    type Target = GraphqlFederationEntityResolverDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

impl<'a> GraphqlFederationEntityResolverDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a GraphqlFederationEntityResolverDefinitionRecord {
        self.ref_
    }
    pub fn subgraph(&self) -> GraphqlSubgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
    pub fn key_fields(&self) -> FieldSet<'a> {
        self.as_ref().key_fields_record.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for &GraphqlFederationEntityResolverDefinitionRecord {
    type Walker<'w>
        = GraphqlFederationEntityResolverDefinition<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        GraphqlFederationEntityResolverDefinition {
            schema: schema.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for GraphqlFederationEntityResolverDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphqlFederationEntityResolverDefinition")
            .field("endpoint", &self.subgraph())
            .field("key_fields", &self.key_fields())
            .finish()
    }
}
