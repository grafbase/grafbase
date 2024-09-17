//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{TypeSystemDirective, TypeSystemDirectiveId},
    prelude::*,
    StringId,
};
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type EnumValue @meta(module: "enum_value") @indexed(id_size: "u32", max_id: "MAX_ID") {
///   name: String!
///   description: String
///   directives: [TypeSystemDirective!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EnumValueRecord {
    pub name_id: StringId,
    pub description_id: Option<StringId>,
    pub directive_ids: Vec<TypeSystemDirectiveId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
#[max(MAX_ID)]
pub struct EnumValueId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct EnumValue<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) id: EnumValueId,
}

impl std::ops::Deref for EnumValue<'_> {
    type Target = EnumValueRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> EnumValue<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a EnumValueRecord {
        &self.schema[self.id]
    }
    pub fn id(&self) -> EnumValueId {
        self.id
    }
    pub fn name(&self) -> &'a str {
        self.name_id.walk(self.schema)
    }
    pub fn description(&self) -> Option<&'a str> {
        self.description_id.walk(self.schema)
    }
    pub fn directives(&self) -> impl Iter<Item = TypeSystemDirective<'a>> + 'a {
        self.as_ref().directive_ids.walk(self.schema)
    }
}

impl Walk<Schema> for EnumValueId {
    type Walker<'a> = EnumValue<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        EnumValue { schema, id: self }
    }
}

impl std::fmt::Debug for EnumValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumValue")
            .field("name", &self.name())
            .field("description", &self.description())
            .field(
                "directives",
                &self.directives().map(|walker| walker.to_string()).collect::<Vec<_>>(),
            )
            .finish()
    }
}
