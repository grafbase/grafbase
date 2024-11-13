//! ===================
//! !!! DO NOT EDIT !!!
//! ===================
//! Generated with: `cargo run -p engine-codegen`
//! Source file: <engine-codegen dir>/domain/operation_plan.graphql
use crate::operation::plan::model::{
    generated::{Plan, PlanId, ResponseModifier, ResponseModifierId},
    prelude::*,
};
use walker::Walk;

/// Generated from:
///
/// ```custom,{.language-graphql}
/// union Executable @id @meta(module: "executable") = Plan | ResponseModifier
/// ```
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ExecutableId {
    Plan(PlanId),
    ResponseModifier(ResponseModifierId),
}

impl std::fmt::Debug for ExecutableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutableId::Plan(variant) => variant.fmt(f),
            ExecutableId::ResponseModifier(variant) => variant.fmt(f),
        }
    }
}

impl From<PlanId> for ExecutableId {
    fn from(value: PlanId) -> Self {
        ExecutableId::Plan(value)
    }
}
impl From<ResponseModifierId> for ExecutableId {
    fn from(value: ResponseModifierId) -> Self {
        ExecutableId::ResponseModifier(value)
    }
}

#[allow(unused)]
impl ExecutableId {
    pub(crate) fn is_plan(&self) -> bool {
        matches!(self, ExecutableId::Plan(_))
    }
    pub(crate) fn as_plan(&self) -> Option<PlanId> {
        match self {
            ExecutableId::Plan(id) => Some(*id),
            _ => None,
        }
    }
    pub(crate) fn is_response_modifier(&self) -> bool {
        matches!(self, ExecutableId::ResponseModifier(_))
    }
    pub(crate) fn as_response_modifier(&self) -> Option<ResponseModifierId> {
        match self {
            ExecutableId::ResponseModifier(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Executable<'a> {
    Plan(Plan<'a>),
    ResponseModifier(ResponseModifier<'a>),
}

impl std::fmt::Debug for Executable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Executable::Plan(variant) => variant.fmt(f),
            Executable::ResponseModifier(variant) => variant.fmt(f),
        }
    }
}

impl<'a> From<Plan<'a>> for Executable<'a> {
    fn from(item: Plan<'a>) -> Self {
        Executable::Plan(item)
    }
}
impl<'a> From<ResponseModifier<'a>> for Executable<'a> {
    fn from(item: ResponseModifier<'a>) -> Self {
        Executable::ResponseModifier(item)
    }
}

impl<'a> Walk<OperationPlanContext<'a>> for ExecutableId {
    type Walker<'w> = Executable<'w> where 'a: 'w ;
    fn walk<'w>(self, ctx: impl Into<OperationPlanContext<'a>>) -> Self::Walker<'w>
    where
        Self: 'w,
        'a: 'w,
    {
        let ctx: OperationPlanContext<'a> = ctx.into();
        match self {
            ExecutableId::Plan(id) => Executable::Plan(id.walk(ctx)),
            ExecutableId::ResponseModifier(id) => Executable::ResponseModifier(id.walk(ctx)),
        }
    }
}

#[allow(unused)]
impl<'a> Executable<'a> {
    pub(crate) fn id(&self) -> ExecutableId {
        match self {
            Executable::Plan(walker) => ExecutableId::Plan(walker.id),
            Executable::ResponseModifier(walker) => ExecutableId::ResponseModifier(walker.id),
        }
    }
    pub(crate) fn is_plan(&self) -> bool {
        matches!(self, Executable::Plan(_))
    }
    pub(crate) fn as_plan(&self) -> Option<Plan<'a>> {
        match self {
            Executable::Plan(item) => Some(*item),
            _ => None,
        }
    }
    pub(crate) fn is_response_modifier(&self) -> bool {
        matches!(self, Executable::ResponseModifier(_))
    }
    pub(crate) fn as_response_modifier(&self) -> Option<ResponseModifier<'a>> {
        match self {
            Executable::ResponseModifier(item) => Some(*item),
            _ => None,
        }
    }
}
