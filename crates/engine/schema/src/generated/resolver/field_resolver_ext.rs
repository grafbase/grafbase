//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    generated::{ExtensionDirective, ExtensionDirectiveId},
    prelude::*,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type FieldResolverExtensionDefinition @meta(module: "resolver/field_resolver_ext") @copy {
///   directive: ExtensionDirective!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct FieldResolverExtensionDefinitionRecord {
    pub directive_id: ExtensionDirectiveId,
}

#[derive(Clone, Copy)]
pub struct FieldResolverExtensionDefinition<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: FieldResolverExtensionDefinitionRecord,
}

impl std::ops::Deref for FieldResolverExtensionDefinition<'_> {
    type Target = FieldResolverExtensionDefinitionRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> FieldResolverExtensionDefinition<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &FieldResolverExtensionDefinitionRecord {
        &self.item
    }
    pub fn directive(&self) -> ExtensionDirective<'a> {
        self.directive_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for FieldResolverExtensionDefinitionRecord {
    type Walker<'w>
        = FieldResolverExtensionDefinition<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        FieldResolverExtensionDefinition {
            schema: schema.into(),
            item: self,
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
