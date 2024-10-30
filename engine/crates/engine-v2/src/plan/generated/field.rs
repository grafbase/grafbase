//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-v2-codegen`
//! Source file: <engine-v2-codegen dir>/domain/operation_plan.graphql
mod argument;
mod data;
mod typename;

use crate::plan::prelude::*;
pub(crate) use argument::*;
pub(crate) use data::*;
pub(crate) use typename::*;
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union Field @indexed(id_size: "u32") @meta(module: "field") @variants(remove_suffix: true) = DataField | TypenameField
/// ```
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) enum FieldRecord {
    Data(DataFieldRecord),
    Typename(TypenameFieldRecord),
}

impl std::fmt::Debug for FieldRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldRecord::Data(variant) => variant.fmt(f),
            FieldRecord::Typename(variant) => variant.fmt(f),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, serde::Serialize, serde::Deserialize, id_derives::Id)]
pub(crate) struct FieldId(std::num::NonZero<u32>);

#[derive(Clone, Copy)]
pub(crate) struct Field<'a> {
    pub(in crate::plan) ctx: PlanContext<'a>,
    pub(in crate::plan) id: FieldId,
}

#[derive(Clone, Copy)]
pub(crate) enum FieldVariant<'a> {
    Data(DataField<'a>),
    Typename(TypenameField<'a>),
}

impl std::fmt::Debug for FieldVariant<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldVariant::Data(variant) => variant.fmt(f),
            FieldVariant::Typename(variant) => variant.fmt(f),
        }
    }
}

impl std::ops::Deref for Field<'_> {
    type Target = FieldRecord;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

#[allow(unused)]
impl<'a> Field<'a> {
    #[allow(clippy::should_implement_trait)]
    pub(crate) fn as_ref(&self) -> &'a FieldRecord {
        &self.ctx.operation_plan[self.id]
    }
    pub(crate) fn id(&self) -> FieldId {
        self.id
    }
    pub(crate) fn variant(&self) -> FieldVariant<'a> {
        let ctx = self.ctx;
        match self.as_ref() {
            FieldRecord::Data(ref item) => FieldVariant::Data(item.walk(ctx)),
            FieldRecord::Typename(ref item) => FieldVariant::Typename(item.walk(ctx)),
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
        Field { ctx, id: self }
    }
}

impl std::fmt::Debug for Field<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.variant().fmt(f)
    }
}
