mod data;
mod typename;

pub(crate) use data::*;
pub(crate) use typename::*;
use walker::Walk;

use crate::plan::PlanFieldId;

use super::QueryContext;

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
