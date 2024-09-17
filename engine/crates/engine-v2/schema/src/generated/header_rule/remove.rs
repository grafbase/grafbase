//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/schema.graphql
use crate::{
    generated::{NameOrPattern, NameOrPatternId},
    prelude::*,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type RemoveHeaderRule @meta(module: "header_rule/remove") @copy {
///   name: NameOrPattern!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct RemoveHeaderRuleRecord {
    pub name_id: NameOrPatternId,
}

#[derive(Clone, Copy)]
pub struct RemoveHeaderRule<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: RemoveHeaderRuleRecord,
}

impl std::ops::Deref for RemoveHeaderRule<'_> {
    type Target = RemoveHeaderRuleRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> RemoveHeaderRule<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &RemoveHeaderRuleRecord {
        &self.item
    }
    pub fn name(&self) -> NameOrPattern<'a> {
        self.name_id.walk(self.schema)
    }
}

impl Walk<Schema> for RemoveHeaderRuleRecord {
    type Walker<'a> = RemoveHeaderRule<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        RemoveHeaderRule { schema, item: self }
    }
}

impl std::fmt::Debug for RemoveHeaderRule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RemoveHeaderRule")
            .field("name", &self.name().to_string())
            .finish()
    }
}
