mod data;
mod typename;

pub(crate) use data::*;
pub(crate) use typename::*;
use walker::Walk;

use super::OperationPlanContext;

#[derive(Clone, Copy)]
pub(crate) enum PlanField<'a> {
    Data(PlanDataField<'a>),
    Typename(PlanTypenameField<'a>),
}

impl std::fmt::Debug for PlanField<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanField::Data(variant) => variant.fmt(f),
            PlanField::Typename(variant) => variant.fmt(f),
        }
    }
}
