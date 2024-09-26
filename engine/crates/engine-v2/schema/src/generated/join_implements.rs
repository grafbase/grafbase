//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{InterfaceDefinition, InterfaceDefinitionId, Subgraph, SubgraphId},
    prelude::*,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type JoinImplementsDefinition @meta(module: "join_implements", derive: ["Clone"]) {
///   interface: InterfaceDefinition!
///   subgraph: Subgraph!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct JoinImplementsDefinitionRecord {
    pub interface_id: InterfaceDefinitionId,
    pub subgraph_id: SubgraphId,
}

#[derive(Clone, Copy)]
pub struct JoinImplementsDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a JoinImplementsDefinitionRecord,
}

impl std::ops::Deref for JoinImplementsDefinition<'_> {
    type Target = JoinImplementsDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

impl<'a> JoinImplementsDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a JoinImplementsDefinitionRecord {
        self.ref_
    }
    pub fn interface(&self) -> InterfaceDefinition<'a> {
        self.interface_id.walk(self.schema)
    }
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
}

impl Walk<Schema> for &JoinImplementsDefinitionRecord {
    type Walker < 'a > = JoinImplementsDefinition < 'a > where Self : 'a ;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        JoinImplementsDefinition { schema, ref_: self }
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
