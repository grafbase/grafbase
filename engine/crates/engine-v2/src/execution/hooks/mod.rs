use runtime::hooks::Hooks;

use crate::Runtime;

use super::{ExecutionContext, PreExecutionContext};

mod authorized;
mod responses;
mod subgraph;

pub(crate) struct RequestHooks<'a, H: Hooks> {
    hooks: &'a H,
    context: &'a H::Context,
}

impl<'a, 'ctx, R: Runtime> From<&'a PreExecutionContext<'ctx, R>> for RequestHooks<'a, R::Hooks>
where
    'ctx: 'a,
{
    fn from(ctx: &'a PreExecutionContext<'ctx, R>) -> Self {
        Self {
            hooks: ctx.engine.runtime.hooks(),
            context: &ctx.hooks_context,
        }
    }
}

impl<'ctx, R: Runtime> From<&ExecutionContext<'ctx, R>> for RequestHooks<'ctx, R::Hooks> {
    fn from(ctx: &ExecutionContext<'ctx, R>) -> Self {
        Self {
            hooks: ctx.engine.runtime.hooks(),
            context: ctx.hooks_context,
        }
    }
}
