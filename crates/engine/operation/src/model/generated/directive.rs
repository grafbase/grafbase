//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation.graphql
use crate::model::{IncludeDirective, IncludeDirectiveRecord, SkipDirective, SkipDirectiveRecord, prelude::*};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Deduplicated
///
/// --------------
/// Generated from:
///
/// ```custom,{.language-graphql}
/// union ExecutableDirective @id @meta(module: "directive") @variants(remove_suffix: "Directive") =
///   | SkipDirective
///   | IncludeDirective
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExecutableDirectiveId {
    Include(IncludeDirectiveRecord),
    Skip(SkipDirectiveRecord),
}

impl std::fmt::Debug for ExecutableDirectiveId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutableDirectiveId::Include(variant) => variant.fmt(f),
            ExecutableDirectiveId::Skip(variant) => variant.fmt(f),
        }
    }
}

impl From<IncludeDirectiveRecord> for ExecutableDirectiveId {
    fn from(value: IncludeDirectiveRecord) -> Self {
        ExecutableDirectiveId::Include(value)
    }
}
impl From<SkipDirectiveRecord> for ExecutableDirectiveId {
    fn from(value: SkipDirectiveRecord) -> Self {
        ExecutableDirectiveId::Skip(value)
    }
}

impl ExecutableDirectiveId {
    pub fn is_include(&self) -> bool {
        matches!(self, ExecutableDirectiveId::Include(_))
    }
    pub fn as_include(&self) -> Option<&IncludeDirectiveRecord> {
        match self {
            ExecutableDirectiveId::Include(item) => Some(item),
            _ => None,
        }
    }
    pub fn is_skip(&self) -> bool {
        matches!(self, ExecutableDirectiveId::Skip(_))
    }
    pub fn as_skip(&self) -> Option<&SkipDirectiveRecord> {
        match self {
            ExecutableDirectiveId::Skip(item) => Some(item),
            _ => None,
        }
    }
}

/// Deduplicated
#[derive(Clone, Copy)]
pub enum ExecutableDirective<'a> {
    Include(IncludeDirective<'a>),
    Skip(SkipDirective<'a>),
}

impl std::fmt::Debug for ExecutableDirective<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutableDirective::Include(variant) => variant.fmt(f),
            ExecutableDirective::Skip(variant) => variant.fmt(f),
        }
    }
}

impl<'a> Walk<OperationContext<'a>> for ExecutableDirectiveId {
    type Walker<'w>
        = ExecutableDirective<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<OperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let ctx: OperationContext<'a> = ctx.into();
        match self {
            ExecutableDirectiveId::Include(item) => ExecutableDirective::Include(item.walk(ctx)),
            ExecutableDirectiveId::Skip(item) => ExecutableDirective::Skip(item.walk(ctx)),
        }
    }
}

impl<'a> ExecutableDirective<'a> {
    pub fn is_include(&self) -> bool {
        matches!(self, ExecutableDirective::Include(_))
    }
    pub fn as_include(&self) -> Option<IncludeDirective<'a>> {
        match self {
            ExecutableDirective::Include(item) => Some(*item),
            _ => None,
        }
    }
    pub fn is_skip(&self) -> bool {
        matches!(self, ExecutableDirective::Skip(_))
    }
    pub fn as_skip(&self) -> Option<SkipDirective<'a>> {
        match self {
            ExecutableDirective::Skip(item) => Some(*item),
            _ => None,
        }
    }
}
