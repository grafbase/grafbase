mod de;
mod debug;
mod ser;

use walker::Walk;

use crate::operation::{BoundVariableDefinitionId, InputValueContext, QueryInputValue, VariableValueRecord};

use super::VariableInputValueRecord;

#[derive(Clone, Copy)]
pub(crate) struct VariableInputValue<'a> {
    pub(super) ctx: InputValueContext<'a>,
    pub(super) ref_: &'a VariableInputValueRecord,
}

#[derive(Clone, Copy)]
pub(crate) enum VariableValue<'a> {
    Undefined,
    Provided(VariableInputValue<'a>),
    DefaultValue(QueryInputValue<'a>),
}

impl<'ctx> Walk<InputValueContext<'ctx>> for BoundVariableDefinitionId {
    type Walker<'w> = VariableValue<'w> where 'ctx: 'w;
    fn walk<'w>(self, ctx: impl Into<InputValueContext<'ctx>>) -> Self::Walker<'w>
    where
        'ctx: 'w,
    {
        let ctx = ctx.into();
        match ctx.variables[self] {
            VariableValueRecord::Undefined => VariableValue::Undefined,
            VariableValueRecord::Provided(id) => VariableValue::Provided(id.walk(ctx)),
            VariableValueRecord::DefaultValue(id) => VariableValue::DefaultValue(id.walk(ctx)),
        }
    }
}

impl<'a> VariableValue<'a> {
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }
}
