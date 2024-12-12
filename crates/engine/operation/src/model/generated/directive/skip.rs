//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation.graphql
use crate::model::{prelude::*, QueryInputValueId};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// type SkipDirective @meta(module: "directive/skip", derive: ["PartialEq", "Eq", "PartialOrd", "Ord", "Hash"]) @copy {
///   condition: QueryInputValueId!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct SkipDirectiveRecord {
    pub condition: QueryInputValueId,
}

#[derive(Clone, Copy)]
pub struct SkipDirective<'a> {
    pub(in crate::model) ctx: OperationContext<'a>,
    pub(in crate::model) item: SkipDirectiveRecord,
}

impl std::ops::Deref for SkipDirective<'_> {
    type Target = SkipDirectiveRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> SkipDirective<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &SkipDirectiveRecord {
        &self.item
    }
}

impl<'a> Walk<OperationContext<'a>> for SkipDirectiveRecord {
    type Walker<'w>
        = SkipDirective<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        SkipDirective {
            ctx: ctx.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for SkipDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkipDirective")
            .field("condition", &self.condition)
            .finish()
    }
}
