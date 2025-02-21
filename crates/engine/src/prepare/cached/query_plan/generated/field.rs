//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/query_plan.graphql
use crate::prepare::cached::query_plan::{
    PartitionDataField, PartitionDataFieldId, PartitionTypenameField, PartitionTypenameFieldId, prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union PartitionField @id @meta(module: "field") @variants(names: ["Data", "Typename"]) =
///   | PartitionDataField
///   | PartitionTypenameField
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum PartitionFieldId {
    Data(PartitionDataFieldId),
    Typename(PartitionTypenameFieldId),
}

impl std::fmt::Debug for PartitionFieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartitionFieldId::Data(variant) => variant.fmt(f),
            PartitionFieldId::Typename(variant) => variant.fmt(f),
        }
    }
}

impl From<PartitionDataFieldId> for PartitionFieldId {
    fn from(value: PartitionDataFieldId) -> Self {
        PartitionFieldId::Data(value)
    }
}
impl From<PartitionTypenameFieldId> for PartitionFieldId {
    fn from(value: PartitionTypenameFieldId) -> Self {
        PartitionFieldId::Typename(value)
    }
}

#[allow(unused)]
impl PartitionFieldId {
    pub(crate) fn is_data(&self) -> bool {
        matches!(self, PartitionFieldId::Data(_))
    }
    pub(crate) fn as_data(&self) -> Option<PartitionDataFieldId> {
        match self {
            PartitionFieldId::Data(id) => Some(*id),
            _ => None,
        }
    }
    pub(crate) fn is_typename(&self) -> bool {
        matches!(self, PartitionFieldId::Typename(_))
    }
    pub(crate) fn as_typename(&self) -> Option<PartitionTypenameFieldId> {
        match self {
            PartitionFieldId::Typename(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum PartitionField<'a> {
    Data(PartitionDataField<'a>),
    Typename(PartitionTypenameField<'a>),
}

impl std::fmt::Debug for PartitionField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartitionField::Data(variant) => variant.fmt(f),
            PartitionField::Typename(variant) => variant.fmt(f),
        }
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for PartitionFieldId {
    type Walker<'w>
        = PartitionField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let ctx: CachedOperationContext<'a> = ctx.into();
        match self {
            PartitionFieldId::Data(id) => PartitionField::Data(id.walk(ctx)),
            PartitionFieldId::Typename(id) => PartitionField::Typename(id.walk(ctx)),
        }
    }
}

#[allow(unused)]
impl<'a> PartitionField<'a> {
    pub(crate) fn id(&self) -> PartitionFieldId {
        match self {
            PartitionField::Data(walker) => PartitionFieldId::Data(walker.id),
            PartitionField::Typename(walker) => PartitionFieldId::Typename(walker.id),
        }
    }
    pub(crate) fn is_data(&self) -> bool {
        matches!(self, PartitionField::Data(_))
    }
    pub(crate) fn as_data(&self) -> Option<PartitionDataField<'a>> {
        match self {
            PartitionField::Data(item) => Some(*item),
            _ => None,
        }
    }
    pub(crate) fn is_typename(&self) -> bool {
        matches!(self, PartitionField::Typename(_))
    }
    pub(crate) fn as_typename(&self) -> Option<PartitionTypenameField<'a>> {
        match self {
            PartitionField::Typename(item) => Some(*item),
            _ => None,
        }
    }
}
