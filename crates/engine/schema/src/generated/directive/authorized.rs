//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{prelude::*, FieldSet, FieldSetId, InputValueSet, SchemaInputValue, SchemaInputValueId};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type AuthorizedDirective @meta(module: "directive/authorized") @indexed(id_size: "u32") {
///   arguments: InputValueSet!
///   fields: FieldSet @field(record_field_name: "fields_id")
///   node: FieldSet
///   metadata: SchemaInputValue
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthorizedDirectiveRecord {
    pub arguments: InputValueSet,
    pub fields_id: Option<FieldSetId>,
    pub node_id: Option<FieldSetId>,
    pub metadata_id: Option<SchemaInputValueId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct AuthorizedDirectiveId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct AuthorizedDirective<'a> {
    pub(crate) schema: &'a Schema,
    pub id: AuthorizedDirectiveId,
}

impl std::ops::Deref for AuthorizedDirective<'_> {
    type Target = AuthorizedDirectiveRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> AuthorizedDirective<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a AuthorizedDirectiveRecord {
        &self.schema[self.id]
    }
    pub fn fields(&self) -> Option<FieldSet<'a>> {
        self.fields_id.walk(self.schema)
    }
    pub fn node(&self) -> Option<FieldSet<'a>> {
        self.node_id.walk(self.schema)
    }
    pub fn metadata(&self) -> Option<SchemaInputValue<'a>> {
        self.metadata_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for AuthorizedDirectiveId {
    type Walker<'w> = AuthorizedDirective<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        AuthorizedDirective {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for AuthorizedDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthorizedDirective")
            .field("arguments", &self.arguments)
            .field("fields", &self.fields())
            .field("node", &self.node())
            .field("metadata", &self.metadata())
            .finish()
    }
}
