//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    FieldSet, FieldSetRecord,
    generated::{
        ArgumentInjection, ArgumentInjectionId, FieldDefinition, FieldDefinitionId, ResolverDefinition,
        ResolverDefinitionId,
    },
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type LookupResolverDefinition @meta(module: "resolver/lookup") @indexed(id_size: "u32") {
///   key: FieldSet!
///   field_definition: FieldDefinition!
///   resolver: ResolverDefinition!
///   guest_batch: Boolean!
///   injections: [ArgumentInjection!]!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct LookupResolverDefinitionRecord {
    pub key_record: FieldSetRecord,
    pub field_definition_id: FieldDefinitionId,
    pub resolver_id: ResolverDefinitionId,
    pub guest_batch: bool,
    pub injection_ids: IdRange<ArgumentInjectionId>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub struct LookupResolverDefinitionId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub struct LookupResolverDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub id: LookupResolverDefinitionId,
}

impl std::ops::Deref for LookupResolverDefinition<'_> {
    type Target = LookupResolverDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> LookupResolverDefinition<'a> {
    /// Prefer using Deref unless you need the 'a lifetime.
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a LookupResolverDefinitionRecord {
        &self.schema[self.id]
    }
    pub fn key(&self) -> FieldSet<'a> {
        self.as_ref().key_record.walk(self.schema)
    }
    pub fn field_definition(&self) -> FieldDefinition<'a> {
        self.field_definition_id.walk(self.schema)
    }
    pub fn resolver(&self) -> ResolverDefinition<'a> {
        self.resolver_id.walk(self.schema)
    }
    pub fn injections(&self) -> impl Iter<Item = ArgumentInjection<'a>> + 'a {
        self.as_ref().injection_ids.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for LookupResolverDefinitionId {
    type Walker<'w>
        = LookupResolverDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        LookupResolverDefinition {
            schema: schema.into(),
            id: self,
        }
    }
}

impl std::fmt::Debug for LookupResolverDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LookupResolverDefinition")
            .field("key", &self.key())
            .field("field_definition", &self.field_definition())
            .field("resolver", &self.resolver())
            .field("guest_batch", &self.guest_batch)
            .field("injections", &self.injections())
            .finish()
    }
}
