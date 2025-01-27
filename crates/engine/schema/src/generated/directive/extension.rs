//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{prelude::*, SchemaInputValue, SchemaInputValueId, StringId};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ExtensionDirective @meta(module: "directive/extension") @indexed(id_size: "u32") {
///   extension_id: ExtensionId!
///   name: String!
///   arguments: SchemaInputValue!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ExtensionDirectiveRecord {
    pub extension_id: ExtensionId,
    pub name_id: StringId,
    pub arguments_id: SchemaInputValueId,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct ExtensionDirectiveId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct ExtensionDirective<'a> {
    pub(crate) schema: &'a Schema,
    pub id: ExtensionDirectiveId,
}

impl std::ops::Deref for ExtensionDirective<'_> {
    type Target = ExtensionDirectiveRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> ExtensionDirective<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a ExtensionDirectiveRecord {
        &self.schema[self.id]
    }
    pub fn name(&self) -> &'a str {
        self.name_id.walk(self.schema)
    }
    pub fn arguments(&self) -> SchemaInputValue<'a> {
        self.arguments_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for ExtensionDirectiveId {
    type Walker<'w>
        = ExtensionDirective<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ExtensionDirective {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for ExtensionDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionDirective")
            .field("extension_id", &self.extension_id)
            .field("name", &self.name())
            .field("arguments", &self.arguments())
            .finish()
    }
}
