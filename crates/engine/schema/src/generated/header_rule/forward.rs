//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{
    StringId,
    generated::{NameOrPattern, NameOrPatternId},
    prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type ForwardHeaderRule @meta(module: "header_rule/forward") @copy {
///   name: NameOrPattern!
///   default: String
///   rename: String
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct ForwardHeaderRuleRecord {
    pub name_id: NameOrPatternId,
    pub default_id: Option<StringId>,
    pub rename_id: Option<StringId>,
}

#[derive(Clone, Copy)]
pub struct ForwardHeaderRule<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: ForwardHeaderRuleRecord,
}

impl std::ops::Deref for ForwardHeaderRule<'_> {
    type Target = ForwardHeaderRuleRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> ForwardHeaderRule<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &ForwardHeaderRuleRecord {
        &self.item
    }
    pub fn name(&self) -> NameOrPattern<'a> {
        self.name_id.walk(self.schema)
    }
    pub fn default(&self) -> Option<&'a str> {
        self.default_id.walk(self.schema)
    }
    pub fn rename(&self) -> Option<&'a str> {
        self.rename_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for ForwardHeaderRuleRecord {
    type Walker<'w>
        = ForwardHeaderRule<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        ForwardHeaderRule {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for ForwardHeaderRule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ForwardHeaderRule")
            .field("name", &self.name())
            .field("default", &self.default())
            .field("rename", &self.rename())
            .finish()
    }
}
