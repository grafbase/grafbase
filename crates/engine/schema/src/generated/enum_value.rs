//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{TypeSystemDirective, TypeSystemDirectiveId},
    prelude::*,
    StringId,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type EnumValue @meta(module: "enum_value") @indexed(id_size: "u32") {
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
pub struct EnumValueId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct EnumValue<'a> {
    pub(crate) schema: &'a Schema,
    pub id: EnumValueId,
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

impl<'a> Walk<&'a Schema> for EnumValueId {
    type Walker<'w>
        = EnumValue<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        EnumValue {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for EnumValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnumValue")
            .field("name", &self.name())
            .field("description", &self.description())
            .field("directives", &self.directives())
            .finish()
    }
}
