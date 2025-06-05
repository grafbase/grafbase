//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{ExtensionDirective, ExtensionDirectiveId, VirtualSubgraph, VirtualSubgraphId},
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
/// type ExtensionResolverDefinition @meta(module: "resolver/extension") @copy {
///   subgraph: VirtualSubgraph!
///   extension_id: ExtensionId!
///   directive: ExtensionDirective
///   guest_batch: Boolean!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct ExtensionResolverDefinitionRecord {
    pub subgraph_id: VirtualSubgraphId,
    pub extension_id: ExtensionId,
    pub directive_id: Option<ExtensionDirectiveId>,
    pub guest_batch: bool,
}

/// There will be one per field contrary to GraphqlRootFieldResolverDefinition
#[derive(Clone, Copy)]
pub struct ExtensionResolverDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: ExtensionResolverDefinitionRecord,
}

impl std::ops::Deref for ExtensionResolverDefinition<'_> {
    type Target = ExtensionResolverDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> ExtensionResolverDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &ExtensionResolverDefinitionRecord {
        &self.item
    }
    pub fn subgraph(&self) -> VirtualSubgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
    pub fn directive(&self) -> Option<ExtensionDirective<'a>> {
        self.directive_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for ExtensionResolverDefinitionRecord {
    type Walker<'w>
        = ExtensionResolverDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ExtensionResolverDefinition {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for ExtensionResolverDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionResolverDefinition")
            .field("subgraph", &self.subgraph())
            .field("extension_id", &self.extension_id)
            .field("directive", &self.directive())
            .field("guest_batch", &self.guest_batch)
            .finish()
    }
}
