//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_solution.graphql
mod argument;
mod data;
mod typename;

use crate::operation::solve::model::prelude::*;
pub(crate) use argument::*;
pub(crate) use data::*;
pub(crate) use typename::*;
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union Field @id @meta(module: "field") @variants(remove_suffix: true) = DataField | TypenameField
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum FieldId {
    Data(DataFieldId),
    Typename(TypenameFieldId),
}

impl std::fmt::Debug for FieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldId::Data(variant) => variant.fmt(f),
            FieldId::Typename(variant) => variant.fmt(f),
        }
    }
}

impl From<DataFieldId> for FieldId {
    fn from(value: DataFieldId) -> Self {
        FieldId::Data(value)
    }
}
impl From<TypenameFieldId> for FieldId {
    fn from(value: TypenameFieldId) -> Self {
        FieldId::Typename(value)
    }
}

#[allow(unused)]
impl FieldId {
    pub(crate) fn is_data(&self) -> bool {
        matches!(self, FieldId::Data(_))
    }
    pub(crate) fn as_data(&self) -> Option<DataFieldId> {
        match self {
            FieldId::Data(id) => Some(*id),
            _ => None,
        }
    }
    pub(crate) fn is_typename(&self) -> bool {
        matches!(self, FieldId::Typename(_))
    }
    pub(crate) fn as_typename(&self) -> Option<TypenameFieldId> {
        match self {
            FieldId::Typename(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Field<'a> {
    Data(DataField<'a>),
    Typename(TypenameField<'a>),
}

impl std::fmt::Debug for Field<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Field::Data(variant) => variant.fmt(f),
            Field::Typename(variant) => variant.fmt(f),
        }
    }
}

impl<'a> From<DataField<'a>> for Field<'a> {
    fn from(item: DataField<'a>) -> Self {
        Field::Data(item)
    }
}
impl<'a> From<TypenameField<'a>> for Field<'a> {
    fn from(item: TypenameField<'a>) -> Self {
        Field::Typename(item)
    }
}

impl<'a> Walk<OperationSolutionContext<'a>> for FieldId {
    type Walker<'w> = Field<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: impl Into<OperationSolutionContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let ctx: OperationSolutionContext<'a> = ctx.into();
        match self {
            FieldId::Data(id) => Field::Data(id.walk(ctx)),
            FieldId::Typename(id) => Field::Typename(id.walk(ctx)),
        }
    }
}

#[allow(unused)]
impl<'a> Field<'a> {
    pub(crate) fn id(&self) -> FieldId {
        match self {
            Field::Data(walker) => FieldId::Data(walker.id),
            Field::Typename(walker) => FieldId::Typename(walker.id),
        }
    }
    pub(crate) fn is_data(&self) -> bool {
        matches!(self, Field::Data(_))
    }
    pub(crate) fn as_data(&self) -> Option<DataField<'a>> {
        match self {
            Field::Data(item) => Some(*item),
            _ => None,
        }
    }
    pub(crate) fn is_typename(&self) -> bool {
        matches!(self, Field::Typename(_))
    }
    pub(crate) fn as_typename(&self) -> Option<TypenameField<'a>> {
        match self {
            Field::Typename(item) => Some(*item),
            _ => None,
        }
    }
}
