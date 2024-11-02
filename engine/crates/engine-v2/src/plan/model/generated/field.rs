//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
mod argument;
mod data;
mod typename;

use crate::plan::model::prelude::*;
pub(crate) use argument::*;
pub(crate) use data::*;
pub(crate) use typename::*;
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union Field @id @indexed(id_size: "u32") @meta(module: "field") @variants(remove_suffix: true) =
///   | DataField
///   | TypenameField
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

impl<'a> Walk<PlanContext<'a>> for FieldId {
    type Walker<'w> = Field<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        match self {
            FieldId::Data(id) => Field::Data(id.walk(ctx)),
            FieldId::Typename(id) => Field::Typename(id.walk(ctx)),
        }
    }
}

#[allow(unused)]
impl Field<'_> {
    pub(crate) fn id(&self) -> FieldId {
        match self {
            Field::Data(walker) => FieldId::Data(walker.id),
            Field::Typename(walker) => FieldId::Typename(walker.id),
        }
    }
}
