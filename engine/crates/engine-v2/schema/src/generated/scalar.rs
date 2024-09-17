//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{TypeSystemDirective, TypeSystemDirectiveId},
    prelude::*,
    ScalarType, StringId,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ScalarDefinition @meta(module: "scalar") @indexed(id_size: "u32", max_id: "MAX_ID") {
///   name: String!
///   ty: ScalarType!
///   description: String
///   specified_by_url: String
///   directives: [TypeSystemDirective!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScalarDefinitionRecord {
    pub name_id: StringId,
    pub ty: ScalarType,
    pub description_id: Option<StringId>,
    pub specified_by_url_id: Option<StringId>,
    pub directive_ids: Vec<TypeSystemDirectiveId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct ScalarDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct ScalarDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) id: ScalarDefinitionId,
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
    pub fn id(&self) -> ScalarDefinitionId {
        self.id
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
}

impl Walk<Schema> for ScalarDefinitionId {
    type Walker<'a> = ScalarDefinition<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        ScalarDefinition { schema, id: self }
    }
}

impl std::fmt::Debug for ScalarDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScalarDefinition")
            .field("name", &self.name())
            .field("ty", &self.ty)
            .field("description", &self.description())
            .field("specified_by_url", &self.specified_by_url())
            .field(
                "directives",
                &self.directives().map(|walker| walker.to_string()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
