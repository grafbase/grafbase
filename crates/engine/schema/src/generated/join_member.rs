//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{ObjectDefinition, ObjectDefinitionId, Subgraph, SubgraphId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

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

impl<'a> Walk<&'a Schema> for JoinMemberDefinitionRecord {
    type Walker<'w>
        = JoinMemberDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        JoinMemberDefinition {
            schema: schema.into(),
            item: self,
        }
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
