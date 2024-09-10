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
/// type DeprecatedDirective
///   @meta(module: "directive/deprecated", derive: ["PartialEq", "Eq", "PartialOrd", "Ord", "Hash"])
///   @copy {
///   reason: String
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct DeprecatedDirectiveRecord {
    pub reason_id: Option<StringId>,
}

#[derive(Clone, Copy)]
pub struct DeprecatedDirective<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) item: DeprecatedDirectiveRecord,
}

impl std::ops::Deref for DeprecatedDirective<'_> {
    type Target = DeprecatedDirectiveRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> DeprecatedDirective<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &DeprecatedDirectiveRecord {
        &self.item
    }
    pub fn reason(&self) -> Option<&'a str> {
        self.reason_id.walk(self.schema)
    }
}

impl Walk<Schema> for DeprecatedDirectiveRecord {
    type Walker<'a> = DeprecatedDirective<'a>;
    fn walk<'a>(self, schema: &'a Schema) -> Self::Walker<'a>
    where
        Self: 'a,
    {
        DeprecatedDirective { schema, item: self }
    }
}

impl std::fmt::Debug for DeprecatedDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeprecatedDirective")
            .field("reason", &self.reason())
            .finish()
    }
}
