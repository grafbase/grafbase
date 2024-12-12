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
/// type IncludeDirective
///   @meta(module: "directive/include", derive: ["PartialEq", "Eq", "PartialOrd", "Ord", "Hash"])
///   @copy {
///   condition: QueryInputValueId!
/// }
/// ```
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub struct IncludeDirectiveRecord {
    pub condition: QueryInputValueId,
}

#[derive(Clone, Copy)]
pub struct IncludeDirective<'a> {
    pub(in crate::model) ctx: OperationContext<'a>,
    pub(in crate::model) item: IncludeDirectiveRecord,
}

impl std::ops::Deref for IncludeDirective<'_> {
    type Target = IncludeDirectiveRecord;
    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<'a> IncludeDirective<'a> {
    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &IncludeDirectiveRecord {
        &self.item
    }
}

impl<'a> Walk<OperationContext<'a>> for IncludeDirectiveRecord {
    type Walker<'w>
        = IncludeDirective<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        IncludeDirective {
            ctx: ctx.into(),
            item: self,
        }
    }
}

impl std::fmt::Debug for IncludeDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IncludeDirective")
            .field("condition", &self.condition)
            .finish()
    }
}
