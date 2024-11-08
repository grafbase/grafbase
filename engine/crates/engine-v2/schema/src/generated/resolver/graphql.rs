//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{GraphqlEndpoint, GraphqlEndpointId},
    prelude::*,
    RequiredFieldSet, RequiredFieldSetId,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type GraphqlRootFieldResolverDefinition @meta(module: "resolver/graphql") @copy {
///   endpoint: GraphqlEndpoint!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct GraphqlRootFieldResolverDefinitionRecord {
    pub endpoint_id: GraphqlEndpointId,
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
    pub fn endpoint(&self) -> GraphqlEndpoint<'a> {
        self.endpoint_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for GraphqlRootFieldResolverDefinitionRecord {
    type Walker<'w> = GraphqlRootFieldResolverDefinition<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: &'a Schema) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        GraphqlRootFieldResolverDefinition { schema, item: self }
    }
}

impl std::fmt::Debug for GraphqlRootFieldResolverDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphqlRootFieldResolverDefinition")
            .field("endpoint", &self.endpoint())
            .finish()
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type GraphqlFederationEntityResolverDefinition @meta(module: "resolver/graphql") @copy {
///   endpoint: GraphqlEndpoint!
///   key_fields: RequiredFieldSet!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct GraphqlFederationEntityResolverDefinitionRecord {
    pub endpoint_id: GraphqlEndpointId,
    pub key_fields_id: RequiredFieldSetId,
}

#[derive(Clone, Copy)]
pub struct GraphqlFederationEntityResolverDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: GraphqlFederationEntityResolverDefinitionRecord,
}

impl std::ops::Deref for GraphqlFederationEntityResolverDefinition<'_> {
    type Target = GraphqlFederationEntityResolverDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> GraphqlFederationEntityResolverDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &GraphqlFederationEntityResolverDefinitionRecord {
        &self.item
    }
    pub fn endpoint(&self) -> GraphqlEndpoint<'a> {
        self.endpoint_id.walk(self.schema)
    }
    pub fn key_fields(&self) -> RequiredFieldSet<'a> {
        self.key_fields_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for GraphqlFederationEntityResolverDefinitionRecord {
    type Walker<'w> = GraphqlFederationEntityResolverDefinition<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: &'a Schema) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        GraphqlFederationEntityResolverDefinition { schema, item: self }
    }
}

impl std::fmt::Debug for GraphqlFederationEntityResolverDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphqlFederationEntityResolverDefinition")
            .field("endpoint", &self.endpoint())
            .field("key_fields", &self.key_fields())
            .finish()
    }
}
