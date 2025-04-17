//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    FieldSet, FieldSetRecord,
    generated::{
        FieldDefinition, FieldDefinitionId, InputValueDefinition, InputValueDefinitionId, ResolverDefinition,
        ResolverDefinitionId,
    },
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type LookupResolverDefinition @meta(module: "resolver/lookup") {
///   key: FieldSet!
///   batch_argument: InputValueDefinition
///   field: FieldDefinition!
///   resolver: ResolverDefinition!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LookupResolverDefinitionRecord {
    pub key_record: FieldSetRecord,
    pub batch_argument_id: Option<InputValueDefinitionId>,
    pub field_id: FieldDefinitionId,
    pub resolver_id: ResolverDefinitionId,
}

#[derive(Clone, Copy)]
pub struct LookupResolverDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a LookupResolverDefinitionRecord,
}

impl std::ops::Deref for LookupResolverDefinition<'_> {
    type Target = LookupResolverDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

impl<'a> LookupResolverDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a LookupResolverDefinitionRecord {
        self.ref_
    }
    pub fn key(&self) -> FieldSet<'a> {
        self.as_ref().key_record.walk(self.schema)
    }
    pub fn batch_argument(&self) -> Option<InputValueDefinition<'a>> {
        self.batch_argument_id.walk(self.schema)
    }
    pub fn field(&self) -> FieldDefinition<'a> {
        self.field_id.walk(self.schema)
    }
    pub fn resolver(&self) -> ResolverDefinition<'a> {
        self.resolver_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for &LookupResolverDefinitionRecord {
    type Walker<'w>
        = LookupResolverDefinition<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        LookupResolverDefinition {
            schema: schema.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for LookupResolverDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LookupResolverDefinition")
            .field("key", &self.key())
            .field("batch_argument", &self.batch_argument())
            .field("field", &self.field())
            .field("resolver", &self.resolver())
            .finish()
    }
}
