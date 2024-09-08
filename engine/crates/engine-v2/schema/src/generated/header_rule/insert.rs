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
/// type InsertHeaderRule @meta(module: "header_rule/insert") @copy {
///   name: String!
///   value: String!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct InsertHeaderRuleRecord {
    pub name_id: StringId,
    pub value_id: StringId,
}

#[derive(Clone, Copy)]
pub struct InsertHeaderRule<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: InsertHeaderRuleRecord,
}

impl std::ops::Deref for InsertHeaderRule<'_> {
    type Target = InsertHeaderRuleRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> InsertHeaderRule<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &InsertHeaderRuleRecord {
        &self.item
    }
    pub fn name(&self) -> &'a str {
        &self.schema[self.as_ref().name_id]
    }
    pub fn value(&self) -> &'a str {
        &self.schema[self.as_ref().value_id]
    }
}

impl Walk<Schema> for InsertHeaderRuleRecord {
    type Walker<'a> = InsertHeaderRule<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        InsertHeaderRule { schema, item: self }
    }
}

impl std::fmt::Debug for InsertHeaderRule<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InsertHeaderRule")
            .field("name", &self.name())
            .field("value", &self.value())
            .finish()
    }
}
