//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/schema.graphql
use crate::{StringId, prelude::*};
#[allow(unused_imports)]
use walker::{Iter, Walk};

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

impl<'a> Walk<&'a Schema> for DeprecatedDirectiveRecord {
    type Walker<'w>
        = DeprecatedDirective<'w>
    where
        'a: 'w;
    fn walk<'w>(self, schema: impl Into<&'a Schema>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        DeprecatedDirective {
            schema: schema.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for DeprecatedDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeprecatedDirective")
            .field("reason", &self.reason())
            .finish()
    }
}
