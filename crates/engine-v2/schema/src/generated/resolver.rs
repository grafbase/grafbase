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

impl ResolverDefinitionRecord {
    pub fn is_graphql_federation_entity(&self) -> bool {
        matches!(self, ResolverDefinitionRecord::GraphqlFederationEntity(_))
    }
    pub fn as_graphql_federation_entity(&self) -> Option<GraphqlFederationEntityResolverDefinitionRecord> {
        match self {
            ResolverDefinitionRecord::GraphqlFederationEntity(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_graphql_root_field(&self) -> bool {
        matches!(self, ResolverDefinitionRecord::GraphqlRootField(_))
    }
    pub fn as_graphql_root_field(&self) -> Option<GraphqlRootFieldResolverDefinitionRecord> {
        match self {
            ResolverDefinitionRecord::GraphqlRootField(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_introspection(&self) -> bool {
        matches!(self, ResolverDefinitionRecord::Introspection)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct ResolverDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct ResolverDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub id: ResolverDefinitionId,
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
    pub fn is_graphql_federation_entity(&self) -> bool {
        matches!(self.variant(), ResolverDefinitionVariant::GraphqlFederationEntity(_))
    }
    pub fn as_graphql_federation_entity(&self) -> Option<GraphqlFederationEntityResolverDefinition<'a>> {
        match self.variant() {
            ResolverDefinitionVariant::GraphqlFederationEntity(item) => Some(item),
            _ => None,
        }
    }
    pub fn is_graphql_root_field(&self) -> bool {
        matches!(self.variant(), ResolverDefinitionVariant::GraphqlRootField(_))
    }
    pub fn as_graphql_root_field(&self) -> Option<GraphqlRootFieldResolverDefinition<'a>> {
        match self.variant() {
            ResolverDefinitionVariant::GraphqlRootField(item) => Some(item),
            _ => None,
        }
    }
    pub fn is_introspection(&self) -> bool {
        matches!(self.variant(), ResolverDefinitionVariant::Introspection)
    }
}

impl<'a> Walk<&'a Schema> for ResolverDefinitionId {
    type Walker<'w> = ResolverDefinition<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ResolverDefinition {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for ResolverDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.variant().fmt(f)
    }
}
