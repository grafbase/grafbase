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
/// union ResolverDefinition
///   @meta(module: "resolver")
///   @variants(empty: ["Introspection"], remove_suffix: true)
///   @indexed(deduplicated: true, id_size: "u32", max_id: "MAX_ID") =
///   | GraphqlRootFieldResolverDefinition
///   | GraphqlFederationEntityResolverDefinition
/// ```
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ResolverDefinitionRecord {
    GraphqlFederationEntity(GraphqlFederationEntityResolverDefinitionRecord),
    GraphqlRootField(GraphqlRootFieldResolverDefinitionRecord),
    Introspection,
}

impl std::fmt::Debug for ResolverDefinitionRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolverDefinitionRecord::GraphqlFederationEntity(variant) => variant.fmt(f),
            ResolverDefinitionRecord::GraphqlRootField(variant) => variant.fmt(f),
            ResolverDefinitionRecord::Introspection => write!(f, "Introspection"),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct ResolverDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct ResolverDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) id: ResolverDefinitionId,
}

#[derive(Clone, Copy)]
pub enum ResolverDefinitionVariant<'a> {
    GraphqlFederationEntity(GraphqlFederationEntityResolverDefinition<'a>),
    GraphqlRootField(GraphqlRootFieldResolverDefinition<'a>),
    Introspection,
}

impl std::fmt::Debug for ResolverDefinitionVariant<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolverDefinitionVariant::GraphqlFederationEntity(variant) => variant.fmt(f),
            ResolverDefinitionVariant::GraphqlRootField(variant) => variant.fmt(f),
            ResolverDefinitionVariant::Introspection => write!(f, "Introspection"),
        }
    }
}

impl std::ops::Deref for ResolverDefinition<'_> {
    type Target = ResolverDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> ResolverDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a ResolverDefinitionRecord {
        &self.schema[self.id]
    }
    pub fn id(&self) -> ResolverDefinitionId {
        self.id
    }
    pub fn variant(&self) -> ResolverDefinitionVariant<'a> {
        let schema = self.schema;
        match self.as_ref() {
            ResolverDefinitionRecord::GraphqlFederationEntity(item) => {
                ResolverDefinitionVariant::GraphqlFederationEntity(item.walk(schema))
            }
            ResolverDefinitionRecord::GraphqlRootField(item) => {
                ResolverDefinitionVariant::GraphqlRootField(item.walk(schema))
            }
            ResolverDefinitionRecord::Introspection => ResolverDefinitionVariant::Introspection,
        }
    }
}

impl Walk<Schema> for ResolverDefinitionId {
    type Walker<'a> = ResolverDefinition<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        ResolverDefinition { schema, id: self }
    }
}

impl std::fmt::Debug for ResolverDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.variant().fmt(f)
    }
}
