//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{InterfaceDefinition, InterfaceDefinitionId, Subgraph, SubgraphId},
    prelude::*,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type JoinImplementsDefinition @meta(module: "join_implements") @copy {
///   interface: InterfaceDefinition!
///   subgraph: Subgraph!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct JoinImplementsDefinitionRecord {
    pub interface_id: InterfaceDefinitionId,
    pub subgraph_id: SubgraphId,
}

#[derive(Clone, Copy)]
pub struct JoinImplementsDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: JoinImplementsDefinitionRecord,
}

impl std::ops::Deref for JoinImplementsDefinition<'_> {
    type Target = JoinImplementsDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> JoinImplementsDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &JoinImplementsDefinitionRecord {
        &self.item
    }
    pub fn interface(&self) -> InterfaceDefinition<'a> {
        self.interface_id.walk(self.schema)
    }
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for JoinImplementsDefinitionRecord {
    type Walker<'w> = JoinImplementsDefinition<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        JoinImplementsDefinition {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for JoinImplementsDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JoinImplementsDefinition")
            .field("interface", &self.interface())
            .field("subgraph", &self.subgraph())
            .finish()
    }
}
