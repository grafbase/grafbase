use std::fmt::Debug;

use schema::Schema;

use crate::{execution::Strings, request::Operation};

pub struct FormatterContext<'a> {
    pub schema: &'a Schema,
    pub opeartion: &'a Operation,
    pub strings: &'a Strings,
}

pub trait FormatterContextHolder {
    fn formatter_context(&self) -> FormatterContext<'_>;
    fn debug<'a, T: ContextAwareDebug>(&'a self, value: &'a T) -> ContextWrapped<'a, T> {
        ContextWrapped {
            ctx: self.formatter_context(),
            value,
        }
    }
}

impl<'a> FormatterContextHolder for FormatterContext<'a> {
    fn formatter_context(&self) -> FormatterContext<'_> {
        FormatterContext {
            schema: self.schema,
            strings: self.strings,
            opeartion: self.opeartion,
        }
    }
}

pub struct ContextWrapped<'a, T> {
    ctx: FormatterContext<'a>,
    value: &'a T,
}

impl<'a, T: ContextAwareDebug> Debug for ContextWrapped<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(&self.ctx, f)
    }
}

pub trait ContextAwareDebug {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<T: ContextAwareDebug> ContextAwareDebug for Vec<T> {
    fn fmt(&self, ctx: &FormatterContext<'_>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter().map(|item| ctx.debug(item))).finish()
    }
}
