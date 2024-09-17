//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{EnumValue, EnumValueId, TypeSystemDirective, TypeSystemDirectiveId},
    prelude::*,
    StringId,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type EnumDefinition @meta(module: "enum_def") @indexed(id_size: "u32", max_id: "MAX_ID") {
///   name: String!
///   description: String
///   values: [EnumValue!]!
///   directives: [TypeSystemDirective!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnumDefinitionRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub value_ids: IdRange<EnumValueId>,
    pub directive_ids: Vec<TypeSystemDirectiveId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct EnumDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct EnumDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) id: EnumDefinitionId,
}

impl std::ops::Deref for EnumDefinition<'_> {
    type Target = EnumDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> EnumDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a EnumDefinitionRecord {
        &self.schema[self.id]
    }
    pub fn id(&self) -> EnumDefinitionId {
        self.id
    }
    pub fn name(&self) -> &'a str {
        self.name_id.walk(self.schema)
    }
    pub fn description(&self) -> Option<&'a str> {
        self.description_id.walk(self.schema)
    }
    pub fn values(&self) -> impl Iter<Item = EnumValue<'a>> + 'a {
        self.value_ids.walk(self.schema)
    }
    pub fn directives(&self) -> impl Iter<Item = TypeSystemDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.schema)
    }
}

impl Walk<Schema> for EnumDefinitionId {
    type Walker<'a> = EnumDefinition<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        EnumDefinition { schema, id: self }
    }
}

impl std::fmt::Debug for EnumDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumDefinition")
            .field("name", &self.name())
            .field("description", &self.description())
            .field(
                "values",
                &self.values().map(|walker| walker.to_string()).collect::<Vec<_>>(),
            )
            .field(
                "directives",
                &self.directives().map(|walker| walker.to_string()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
