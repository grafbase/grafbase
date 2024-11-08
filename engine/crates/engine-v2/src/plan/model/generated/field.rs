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
/// union PlanField @id @indexed(id_size: "u32") @meta(module: "field") @variants(remove_suffix: true) =
///   | DataPlanField
///   | TypenamePlanField
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum PlanFieldId {
    Data(DataPlanFieldId),
    Typename(TypenamePlanFieldId),
}

impl std::fmt::Debug for PlanFieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanFieldId::Data(variant) => variant.fmt(f),
            PlanFieldId::Typename(variant) => variant.fmt(f),
        }
    }
}

impl From<DataPlanFieldId> for PlanFieldId {
    fn from(value: DataPlanFieldId) -> Self {
        PlanFieldId::Data(value)
    }
}
impl From<TypenamePlanFieldId> for PlanFieldId {
    fn from(value: TypenamePlanFieldId) -> Self {
        PlanFieldId::Typename(value)
    }
}

#[derive(Clone, Copy)]
pub(crate) enum PlanField<'a> {
    Data(DataPlanField<'a>),
    Typename(TypenamePlanField<'a>),
}

impl std::fmt::Debug for PlanField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanField::Data(variant) => variant.fmt(f),
            PlanField::Typename(variant) => variant.fmt(f),
        }
    }
}

impl<'a> Walk<PlanContext<'a>> for PlanFieldId {
    type Walker<'w> = PlanField<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: PlanContext<'a>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        match self {
            PlanFieldId::Data(id) => PlanField::Data(id.walk(ctx)),
            PlanFieldId::Typename(id) => PlanField::Typename(id.walk(ctx)),
        }
    }
}

#[allow(unused)]
impl PlanField<'_> {
    pub(crate) fn id(&self) -> PlanFieldId {
        match self {
            PlanField::Data(walker) => PlanFieldId::Data(walker.id),
            PlanField::Typename(walker) => PlanFieldId::Typename(walker.id),
        }
    }
}
