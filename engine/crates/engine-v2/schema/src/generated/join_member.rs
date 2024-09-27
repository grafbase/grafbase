//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{ObjectDefinition, ObjectDefinitionId, Subgraph, SubgraphId},
    prelude::*,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type JoinMemberDefinition @meta(module: "join_member") @copy {
///   member: ObjectDefinition!
///   subgraph: Subgraph!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct JoinMemberDefinitionRecord {
    pub member_id: ObjectDefinitionId,
    pub subgraph_id: SubgraphId,
}

#[derive(Clone, Copy)]
pub struct JoinMemberDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: JoinMemberDefinitionRecord,
}

impl std::ops::Deref for JoinMemberDefinition<'_> {
    type Target = JoinMemberDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> JoinMemberDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &JoinMemberDefinitionRecord {
        &self.item
    }
    pub fn member(&self) -> ObjectDefinition<'a> {
        self.member_id.walk(self.schema)
    }
    pub fn subgraph(&self) -> Subgraph<'a> {
        self.subgraph_id.walk(self.schema)
    }
}

impl Walk<Schema> for JoinMemberDefinitionRecord {
    type Walker<'a> = JoinMemberDefinition<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        JoinMemberDefinition { schema, item: self }
    }
}

impl std::fmt::Debug for JoinMemberDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JoinMemberDefinition")
            .field("member", &self.member())
            .field("subgraph", &self.subgraph())
            .finish()
    }
}
