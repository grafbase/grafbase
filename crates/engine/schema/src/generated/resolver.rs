//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
mod field_resolver_ext;
mod graphql;
mod lookup;
mod selection_set_resolver_ext;

use crate::prelude::*;
pub use field_resolver_ext::*;
pub use graphql::*;
pub use lookup::*;
pub use selection_set_resolver_ext::*;
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union ResolverDefinition
///   @meta(module: "resolver")
///   @variants(
///     empty: ["Introspection"]
///     names: [
///       "GraphqlRootField"
///       "GraphqlFederationEntity"
///       "FieldResolverExtension"
///       "SelectionSetResolverExtension"
///       "Lookup"
///     ]
///   )
///   @indexed(deduplicated: true, id_size: "u32") =
///   | GraphqlRootFieldResolverDefinition
///   | GraphqlFederationEntityResolverDefinition
///   | FieldResolverExtensionDefinition
///   | SelectionSetResolverExtensionDefinition
///   | LookupResolverDefinition
/// ```
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ResolverDefinitionRecord {
    FieldResolverExtension(FieldResolverExtensionDefinitionRecord),
    GraphqlFederationEntity(GraphqlFederationEntityResolverDefinitionRecord),
    GraphqlRootField(GraphqlRootFieldResolverDefinitionRecord),
    Introspection,
    Lookup(LookupResolverDefinitionId),
    SelectionSetResolverExtension(SelectionSetResolverExtensionDefinitionRecord),
}

impl std::fmt::Debug for ResolverDefinitionRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolverDefinitionRecord::FieldResolverExtension(variant) => variant.fmt(f),
            ResolverDefinitionRecord::GraphqlFederationEntity(variant) => variant.fmt(f),
            ResolverDefinitionRecord::GraphqlRootField(variant) => variant.fmt(f),
            ResolverDefinitionRecord::Introspection => write!(f, "Introspection"),
            ResolverDefinitionRecord::Lookup(variant) => variant.fmt(f),
            ResolverDefinitionRecord::SelectionSetResolverExtension(variant) => variant.fmt(f),
        }
    }
}

impl From<FieldResolverExtensionDefinitionRecord> for ResolverDefinitionRecord {
    fn from(value: FieldResolverExtensionDefinitionRecord) -> Self {
        ResolverDefinitionRecord::FieldResolverExtension(value)
    }
}
impl From<GraphqlFederationEntityResolverDefinitionRecord> for ResolverDefinitionRecord {
    fn from(value: GraphqlFederationEntityResolverDefinitionRecord) -> Self {
        ResolverDefinitionRecord::GraphqlFederationEntity(value)
    }
}
impl From<GraphqlRootFieldResolverDefinitionRecord> for ResolverDefinitionRecord {
    fn from(value: GraphqlRootFieldResolverDefinitionRecord) -> Self {
        ResolverDefinitionRecord::GraphqlRootField(value)
    }
}
impl From<LookupResolverDefinitionId> for ResolverDefinitionRecord {
    fn from(value: LookupResolverDefinitionId) -> Self {
        ResolverDefinitionRecord::Lookup(value)
    }
}
impl From<SelectionSetResolverExtensionDefinitionRecord> for ResolverDefinitionRecord {
    fn from(value: SelectionSetResolverExtensionDefinitionRecord) -> Self {
        ResolverDefinitionRecord::SelectionSetResolverExtension(value)
    }
}

impl ResolverDefinitionRecord {
    pub fn is_field_resolver_extension(&self) -> bool {
        matches!(self, ResolverDefinitionRecord::FieldResolverExtension(_))
    }
    pub fn as_field_resolver_extension(&self) -> Option<&FieldResolverExtensionDefinitionRecord> {
        match self {
            ResolverDefinitionRecord::FieldResolverExtension(item) => Some(item),
            _ => None,
        }
    }
    pub fn is_graphql_federation_entity(&self) -> bool {
        matches!(self, ResolverDefinitionRecord::GraphqlFederationEntity(_))
    }
    pub fn as_graphql_federation_entity(&self) -> Option<&GraphqlFederationEntityResolverDefinitionRecord> {
        match self {
            ResolverDefinitionRecord::GraphqlFederationEntity(item) => Some(item),
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
    pub fn is_lookup(&self) -> bool {
        matches!(self, ResolverDefinitionRecord::Lookup(_))
    }
    pub fn as_lookup(&self) -> Option<LookupResolverDefinitionId> {
        match self {
            ResolverDefinitionRecord::Lookup(id) => Some(*id),
            _ => None,
        }
    }
    pub fn is_selection_set_resolver_extension(&self) -> bool {
        matches!(self, ResolverDefinitionRecord::SelectionSetResolverExtension(_))
    }
    pub fn as_selection_set_resolver_extension(&self) -> Option<SelectionSetResolverExtensionDefinitionRecord> {
        match self {
            ResolverDefinitionRecord::SelectionSetResolverExtension(item) => Some(*item),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ResolverDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct ResolverDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub id: ResolverDefinitionId,
}

#[derive(Clone, Copy)]
pub enum ResolverDefinitionVariant<'a> {
    FieldResolverExtension(FieldResolverExtensionDefinition<'a>),
    GraphqlFederationEntity(GraphqlFederationEntityResolverDefinition<'a>),
    GraphqlRootField(GraphqlRootFieldResolverDefinition<'a>),
    Introspection(&'a Schema),
    Lookup(LookupResolverDefinition<'a>),
    SelectionSetResolverExtension(SelectionSetResolverExtensionDefinition<'a>),
}

impl std::fmt::Debug for ResolverDefinitionVariant<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolverDefinitionVariant::FieldResolverExtension(variant) => variant.fmt(f),
            ResolverDefinitionVariant::GraphqlFederationEntity(variant) => variant.fmt(f),
            ResolverDefinitionVariant::GraphqlRootField(variant) => variant.fmt(f),
            ResolverDefinitionVariant::Introspection(_) => write!(f, "Introspection"),
            ResolverDefinitionVariant::Lookup(variant) => variant.fmt(f),
            ResolverDefinitionVariant::SelectionSetResolverExtension(variant) => variant.fmt(f),
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
            ResolverDefinitionRecord::FieldResolverExtension(item) => {
                ResolverDefinitionVariant::FieldResolverExtension(item.walk(schema))
            }
            ResolverDefinitionRecord::GraphqlFederationEntity(item) => {
                ResolverDefinitionVariant::GraphqlFederationEntity(item.walk(schema))
            }
            ResolverDefinitionRecord::GraphqlRootField(item) => {
                ResolverDefinitionVariant::GraphqlRootField(item.walk(schema))
            }
            ResolverDefinitionRecord::Introspection => ResolverDefinitionVariant::Introspection(schema),
            ResolverDefinitionRecord::Lookup(id) => ResolverDefinitionVariant::Lookup(id.walk(schema)),
            ResolverDefinitionRecord::SelectionSetResolverExtension(item) => {
                ResolverDefinitionVariant::SelectionSetResolverExtension(item.walk(schema))
            }
        }
    }
    pub fn is_field_resolver_extension(&self) -> bool {
        matches!(self.variant(), ResolverDefinitionVariant::FieldResolverExtension(_))
    }
    pub fn as_field_resolver_extension(&self) -> Option<FieldResolverExtensionDefinition<'a>> {
        match self.variant() {
            ResolverDefinitionVariant::FieldResolverExtension(item) => Some(item),
            _ => None,
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
        matches!(self.variant(), ResolverDefinitionVariant::Introspection(_))
    }
    pub fn is_lookup(&self) -> bool {
        matches!(self.variant(), ResolverDefinitionVariant::Lookup(_))
    }
    pub fn as_lookup(&self) -> Option<LookupResolverDefinition<'a>> {
        match self.variant() {
            ResolverDefinitionVariant::Lookup(item) => Some(item),
            _ => None,
        }
    }
    pub fn is_selection_set_resolver_extension(&self) -> bool {
        matches!(
            self.variant(),
            ResolverDefinitionVariant::SelectionSetResolverExtension(_)
        )
    }
    pub fn as_selection_set_resolver_extension(&self) -> Option<SelectionSetResolverExtensionDefinition<'a>> {
        match self.variant() {
            ResolverDefinitionVariant::SelectionSetResolverExtension(item) => Some(item),
            _ => None,
        }
    }
}

impl<'a> Walk<&'a Schema> for ResolverDefinitionId {
    type Walker<'w>
        = ResolverDefinition<'w>
    where
        'a: 'w;
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
