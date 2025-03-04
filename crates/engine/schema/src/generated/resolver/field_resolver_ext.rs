//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{ExtensionDirective, ExtensionDirectiveId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type FieldResolverExtensionDefinition @meta(module: "resolver/field_resolver_ext") {
///   directive: ExtensionDirective!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FieldResolverExtensionDefinitionRecord {
    pub directive_id: ExtensionDirectiveId,
}

#[derive(Clone, Copy)]
pub struct FieldResolverExtensionDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) ref_: &'a FieldResolverExtensionDefinitionRecord,
}

impl std::ops::Deref for FieldResolverExtensionDefinition<'_> {
    type Target = FieldResolverExtensionDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        self.ref_
    }
}

impl<'a> FieldResolverExtensionDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &'a FieldResolverExtensionDefinitionRecord {
        self.ref_
    }
    pub fn directive(&self) -> ExtensionDirective<'a> {
        self.directive_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for &FieldResolverExtensionDefinitionRecord {
    type Walker<'w>
        = FieldResolverExtensionDefinition<'w>
    where
        Self: 'w,
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        FieldResolverExtensionDefinition {
            schema: schema.into(),
            ref_: self,
        }
    }
}

impl std::fmt::Debug for FieldResolverExtensionDefinition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldResolverExtensionDefinition")
            .field("directive", &self.directive())
            .finish()
    }
}
