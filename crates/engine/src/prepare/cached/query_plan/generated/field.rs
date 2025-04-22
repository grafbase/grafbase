//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/query_plan.graphql
use crate::prepare::cached::query_plan::{
    DataField, DataFieldId, LookupField, LookupFieldId, TypenameField, TypenameFieldId, prelude::*,
};
#[allow(unused_imports)]
use walker::{Iter, Walk};

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union PartitionField @id @meta(module: "field") @variants(remove_suffix: "Field") =
///   | DataField
///   | TypenameField
///   | LookupField
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum PartitionFieldId {
    Data(DataFieldId),
    Lookup(LookupFieldId),
    Typename(TypenameFieldId),
}

impl std::fmt::Debug for PartitionFieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartitionFieldId::Data(variant) => variant.fmt(f),
            PartitionFieldId::Lookup(variant) => variant.fmt(f),
            PartitionFieldId::Typename(variant) => variant.fmt(f),
        }
    }
}

impl From<DataFieldId> for PartitionFieldId {
    fn from(value: DataFieldId) -> Self {
        PartitionFieldId::Data(value)
    }
}
impl From<LookupFieldId> for PartitionFieldId {
    fn from(value: LookupFieldId) -> Self {
        PartitionFieldId::Lookup(value)
    }
}
impl From<TypenameFieldId> for PartitionFieldId {
    fn from(value: TypenameFieldId) -> Self {
        PartitionFieldId::Typename(value)
    }
}

#[allow(unused)]
impl PartitionFieldId {
    pub(crate) fn is_data(&self) -> bool {
        matches!(self, PartitionFieldId::Data(_))
    }
    pub(crate) fn as_data(&self) -> Option<DataFieldId> {
        match self {
            PartitionFieldId::Data(id) => Some(*id),
            _ => None,
        }
    }
    pub(crate) fn is_lookup(&self) -> bool {
        matches!(self, PartitionFieldId::Lookup(_))
    }
    pub(crate) fn as_lookup(&self) -> Option<LookupFieldId> {
        match self {
            PartitionFieldId::Lookup(id) => Some(*id),
            _ => None,
        }
    }
    pub(crate) fn is_typename(&self) -> bool {
        matches!(self, PartitionFieldId::Typename(_))
    }
    pub(crate) fn as_typename(&self) -> Option<TypenameFieldId> {
        match self {
            PartitionFieldId::Typename(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum PartitionField<'a> {
    Data(DataField<'a>),
    Lookup(LookupField<'a>),
    Typename(TypenameField<'a>),
}

impl std::fmt::Debug for PartitionField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartitionField::Data(variant) => variant.fmt(f),
            PartitionField::Lookup(variant) => variant.fmt(f),
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
            PartitionFieldId::Lookup(id) => PartitionField::Lookup(id.walk(ctx)),
            PartitionFieldId::Typename(id) => PartitionField::Typename(id.walk(ctx)),
        }
    }
}

#[allow(unused)]
impl<'a> PartitionField<'a> {
    pub(crate) fn id(&self) -> PartitionFieldId {
        match self {
            PartitionField::Data(walker) => PartitionFieldId::Data(walker.id),
            PartitionField::Lookup(walker) => PartitionFieldId::Lookup(walker.id),
            PartitionField::Typename(walker) => PartitionFieldId::Typename(walker.id),
        }
    }
    pub(crate) fn is_data(&self) -> bool {
        matches!(self, PartitionField::Data(_))
    }
    pub(crate) fn as_data(&self) -> Option<DataField<'a>> {
        match self {
            PartitionField::Data(item) => Some(*item),
            _ => None,
        }
    }
    pub(crate) fn is_lookup(&self) -> bool {
        matches!(self, PartitionField::Lookup(_))
    }
    pub(crate) fn as_lookup(&self) -> Option<LookupField<'a>> {
        match self {
            PartitionField::Lookup(item) => Some(*item),
            _ => None,
        }
    }
    pub(crate) fn is_typename(&self) -> bool {
        matches!(self, PartitionField::Typename(_))
    }
    pub(crate) fn as_typename(&self) -> Option<TypenameField<'a>> {
        match self {
            PartitionField::Typename(item) => Some(*item),
            _ => None,
        }
    }
}

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union DataOrLookupField @id @meta(module: "field") @variants(remove_suffix: "Field") = DataField | LookupField
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DataOrLookupFieldId {
    Data(DataFieldId),
    Lookup(LookupFieldId),
}

impl std::fmt::Debug for DataOrLookupFieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataOrLookupFieldId::Data(variant) => variant.fmt(f),
            DataOrLookupFieldId::Lookup(variant) => variant.fmt(f),
        }
    }
}

impl From<DataFieldId> for DataOrLookupFieldId {
    fn from(value: DataFieldId) -> Self {
        DataOrLookupFieldId::Data(value)
    }
}
impl From<LookupFieldId> for DataOrLookupFieldId {
    fn from(value: LookupFieldId) -> Self {
        DataOrLookupFieldId::Lookup(value)
    }
}

#[allow(unused)]
impl DataOrLookupFieldId {
    pub(crate) fn is_data(&self) -> bool {
        matches!(self, DataOrLookupFieldId::Data(_))
    }
    pub(crate) fn as_data(&self) -> Option<DataFieldId> {
        match self {
            DataOrLookupFieldId::Data(id) => Some(*id),
            _ => None,
        }
    }
    pub(crate) fn is_lookup(&self) -> bool {
        matches!(self, DataOrLookupFieldId::Lookup(_))
    }
    pub(crate) fn as_lookup(&self) -> Option<LookupFieldId> {
        match self {
            DataOrLookupFieldId::Lookup(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum DataOrLookupField<'a> {
    Data(DataField<'a>),
    Lookup(LookupField<'a>),
}

impl std::fmt::Debug for DataOrLookupField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataOrLookupField::Data(variant) => variant.fmt(f),
            DataOrLookupField::Lookup(variant) => variant.fmt(f),
        }
    }
}

impl<'a> Walk<CachedOperationContext<'a>> for DataOrLookupFieldId {
    type Walker<'w>
        = DataOrLookupField<'w>
    where
        'a: 'w;
    fn walk<'w>(self, ctx: impl Into<CachedOperationContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let ctx: CachedOperationContext<'a> = ctx.into();
        match self {
            DataOrLookupFieldId::Data(id) => DataOrLookupField::Data(id.walk(ctx)),
            DataOrLookupFieldId::Lookup(id) => DataOrLookupField::Lookup(id.walk(ctx)),
        }
    }
}

#[allow(unused)]
impl<'a> DataOrLookupField<'a> {
    pub(crate) fn id(&self) -> DataOrLookupFieldId {
        match self {
            DataOrLookupField::Data(walker) => DataOrLookupFieldId::Data(walker.id),
            DataOrLookupField::Lookup(walker) => DataOrLookupFieldId::Lookup(walker.id),
        }
    }
    pub(crate) fn is_data(&self) -> bool {
        matches!(self, DataOrLookupField::Data(_))
    }
    pub(crate) fn as_data(&self) -> Option<DataField<'a>> {
        match self {
            DataOrLookupField::Data(item) => Some(*item),
            _ => None,
        }
    }
    pub(crate) fn is_lookup(&self) -> bool {
        matches!(self, DataOrLookupField::Lookup(_))
    }
    pub(crate) fn as_lookup(&self) -> Option<LookupField<'a>> {
        match self {
            DataOrLookupField::Lookup(item) => Some(*item),
            _ => None,
        }
    }
}
