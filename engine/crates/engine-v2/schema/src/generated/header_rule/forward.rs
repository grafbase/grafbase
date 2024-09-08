//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{NameOrPattern, NameOrPatternId},
    prelude::*,
    StringId,
};
use walker::Walk;

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
        self.as_ref().name_id.walk(self.schema)
    }
    pub fn default(&self) -> Option<&'a str> {
        self.as_ref().default_id.map(|id| self.schema[id].as_ref())
    }
    pub fn rename(&self) -> Option<&'a str> {
        self.as_ref().rename_id.map(|id| self.schema[id].as_ref())
    }
}

impl Walk<Schema> for ForwardHeaderRuleRecord {
    type Walker<'a> = ForwardHeaderRule<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        ForwardHeaderRule { schema, item: self }
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
