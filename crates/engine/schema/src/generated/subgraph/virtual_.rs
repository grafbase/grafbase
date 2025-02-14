//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    StringId,
    generated::{ExtensionDirective, ExtensionDirectiveId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Virtual subgraphs have no dedicated support on the engine side, everything is resolved through extensions.
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// type VirtualSubgraph @meta(module: "subgraph/virtual_") @indexed(id_size: "u16") {
///   subgraph_name: String!
///   "Schema directives applied by the given subgraph"
///   schema_directives: [ExtensionDirective!]! @vec
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VirtualSubgraphRecord {
    pub subgraph_name_id: StringId,
    /// Schema directives applied by the given subgraph
    pub schema_directive_ids: Vec<ExtensionDirectiveId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct VirtualSubgraphId(std::num::NonZero<u16>);

/// Virtual subgraphs have no dedicated support on the engine side, everything is resolved through extensions.
#[derive(Clone, Copy)]
pub struct VirtualSubgraph<'a> {
    pub(crate) schema: &'a Schema,
    pub id: VirtualSubgraphId,
}

impl std::ops::Deref for VirtualSubgraph<'_> {
    type Target = VirtualSubgraphRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> VirtualSubgraph<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a VirtualSubgraphRecord {
        &self.schema[self.id]
    }
    pub fn subgraph_name(&self) -> &'a str {
        self.subgraph_name_id.walk(self.schema)
    }
    /// Schema directives applied by the given subgraph
    pub fn schema_directives(&self) -> impl Iter<Item = ExtensionDirective<'a>> + 'a {
        self.as_ref().schema_directive_ids.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for VirtualSubgraphId {
    type Walker<'w>
        = VirtualSubgraph<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        VirtualSubgraph {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for VirtualSubgraph<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualSubgraph")
            .field("subgraph_name", &self.subgraph_name())
            .field("schema_directives", &self.schema_directives())
            .finish()
    }
}
