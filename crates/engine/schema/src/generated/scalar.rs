//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    ScalarType, StringId,
    generated::{Subgraph, SubgraphId, TypeSystemDirective, TypeSystemDirectiveId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ScalarDefinition @meta(module: "scalar") @indexed(id_size: "u32") {
///   name: String!
///   ty: ScalarType!
///   description: String
///   specified_by_url: String
///   directives: [TypeSystemDirective!]!
///   exists_in_subgraphs: [Subgraph!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScalarDefinitionRecord {
    pub name_id: StringId,
    pub ty: ScalarType,
    pub description_id: Option<StringId>,
    pub specified_by_url_id: Option<StringId>,
    pub directive_ids: Vec<TypeSystemDirectiveId>,
    pub exists_in_subgraph_ids: Vec<SubgraphId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ScalarDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct ScalarDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub id: ScalarDefinitionId,
}

impl std::ops::Deref for ScalarDefinition<'_> {
    type Target = ScalarDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> ScalarDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a ScalarDefinitionRecord {
        &self.schema[self.id]
    }
    pub fn name(&self) -> &'a str {
        self.name_id.walk(self.schema)
    }
    pub fn description(&self) -> Option<&'a str> {
        self.description_id.walk(self.schema)
    }
    pub fn specified_by_url(&self) -> Option<&'a str> {
        self.specified_by_url_id.walk(self.schema)
    }
    pub fn directives(&self) -> impl Iter<Item = TypeSystemDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.schema)
    }
    pub fn exists_in_subgraphs(&self) -> impl Iter<Item = Subgraph<'a>> + 'a {
        self.as_ref().exists_in_subgraph_ids.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for ScalarDefinitionId {
    type Walker<'w>
        = ScalarDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ScalarDefinition {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for ScalarDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScalarDefinition")
            .field("name", &self.name())
            .field("ty", &self.ty)
            .field("description", &self.description())
            .field("specified_by_url", &self.specified_by_url())
            .field("directives", &self.directives())
            .field("exists_in_subgraphs", &self.exists_in_subgraphs())
            .finish()
    }
}
