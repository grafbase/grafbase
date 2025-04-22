mod de;
mod debug;
mod ser;

use walker::Walk;

use crate::{InputValueContext, QueryInputValue, VariableDefinitionId, VariableValueRecord};

use super::VariableInputValueRecord;

#[derive(Clone, Copy)]
pub struct VariableInputValue<'a> {
    pub(super) ctx: InputValueContext<'a>,
    pub(super) ref_: &'a VariableInputValueRecord,
}

#[derive(Clone, Copy)]
pub enum VariableValue<'a> {
    Undefined,
    Provided(VariableInputValue<'a>),
    DefaultValue(QueryInputValue<'a>),
}

impl<'ctx> Walk<InputValueContext<'ctx>> for VariableDefinitionId {
    type Walker<'w>
        = VariableValue<'w>
    where
        'ctx: 'w;
    fn walk<'w>(self, ctx: impl Into<InputValueContext<'ctx>>) -> Self::Walker<'w>
    where
        'ctx: 'w,
    {
        let ctx: InputValueContext<'ctx> = ctx.into();
        match ctx.variables[self] {
            VariableValueRecord::Undefined => VariableValue::Undefined,
            VariableValueRecord::Provided(id) => VariableValue::Provided(id.walk(ctx)),
            VariableValueRecord::DefaultValue(id) => VariableValue::DefaultValue(id.walk(ctx)),
        }
    }
}

impl VariableValue<'_> {
    pub fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }
}
