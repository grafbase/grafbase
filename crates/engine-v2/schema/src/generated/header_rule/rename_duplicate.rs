//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{prelude::*, StringId};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type RenameDuplicateHeaderRule @meta(module: "header_rule/rename_duplicate") @copy {
///   name: String!
///   default: String
///   rename: String!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct RenameDuplicateHeaderRuleRecord {
    pub name_id: StringId,
    pub default_id: Option<StringId>,
    pub rename_id: StringId,
}

#[derive(Clone, Copy)]
pub struct RenameDuplicateHeaderRule<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: RenameDuplicateHeaderRuleRecord,
}

impl std::ops::Deref for RenameDuplicateHeaderRule<'_> {
    type Target = RenameDuplicateHeaderRuleRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> RenameDuplicateHeaderRule<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &RenameDuplicateHeaderRuleRecord {
        &self.item
    }
    pub fn name(&self) -> &'a str {
        self.name_id.walk(self.schema)
    }
    pub fn default(&self) -> Option<&'a str> {
        self.default_id.walk(self.schema)
    }
    pub fn rename(&self) -> &'a str {
        self.rename_id.walk(self.schema)
    }
}

impl<'a> Walk<&'a Schema> for RenameDuplicateHeaderRuleRecord {
    type Walker<'w> = RenameDuplicateHeaderRule<'w> where 'a: 'w ;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        RenameDuplicateHeaderRule {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for RenameDuplicateHeaderRule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenameDuplicateHeaderRule")
            .field("name", &self.name())
            .field("default", &self.default())
            .field("rename", &self.rename())
            .finish()
    }
}
