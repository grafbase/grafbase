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

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type SelectionSetResolverExtensionDefinition @meta(module: "resolver/selection_set_ext") @copy {
///   subgraph: VirtualSubgraph!
///   extension_id: ExtensionId!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct SelectionSetResolverExtensionDefinitionRecord {
    pub subgraph_id: VirtualSubgraphId,
    pub extension_id: ExtensionId,
}

#[derive(Clone, Copy)]
pub struct SelectionSetResolverExtensionDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: SelectionSetResolverExtensionDefinitionRecord,
}

impl std::ops::Deref for SelectionSetResolverExtensionDefinition<'_> {
    type Target = SelectionSetResolverExtensionDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> SelectionSetResolverExtensionDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &SelectionSetResolverExtensionDefinitionRecord {
        &self.item
    }
    pub fn subgraph(&self) -> VirtualSubgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for SelectionSetResolverExtensionDefinitionRecord {
    type Walker<'w>
        = SelectionSetResolverExtensionDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        SelectionSetResolverExtensionDefinition {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for SelectionSetResolverExtensionDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelectionSetResolverExtensionDefinition")
            .field("subgraph", &self.subgraph())
            .field("extension_id", &self.extension_id)
            .finish()
    }
}
