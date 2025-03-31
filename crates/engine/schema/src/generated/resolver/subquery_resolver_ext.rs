//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{VirtualSubgraph, VirtualSubgraphId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// There will be one per field contrary to GraphqlRootFieldResolverDefinition
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// type SubQueryResolverExtensionDefinition @meta(module: "resolver/subquery_resolver_ext") @copy {
///   subgraph: VirtualSubgraph!
///   extension_id: ExtensionId!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct SubQueryResolverExtensionDefinitionRecord {
    pub subgraph_id: VirtualSubgraphId,
    pub extension_id: ExtensionId,
}

/// There will be one per field contrary to GraphqlRootFieldResolverDefinition
#[derive(Clone, Copy)]
pub struct SubQueryResolverExtensionDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: SubQueryResolverExtensionDefinitionRecord,
}

impl std::ops::Deref for SubQueryResolverExtensionDefinition<'_> {
    type Target = SubQueryResolverExtensionDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> SubQueryResolverExtensionDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &SubQueryResolverExtensionDefinitionRecord {
        &self.item
    }
    pub fn subgraph(&self) -> VirtualSubgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for SubQueryResolverExtensionDefinitionRecord {
    type Walker<'w>
        = SubQueryResolverExtensionDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        SubQueryResolverExtensionDefinition {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for SubQueryResolverExtensionDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubQueryResolverExtensionDefinition")
            .field("subgraph", &self.subgraph())
            .field("extension_id", &self.extension_id)
            .finish()
    }
}
