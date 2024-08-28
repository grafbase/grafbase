use runtime::hooks::Hooks;

use crate::Runtime;

use super::{ExecutionContext, PreExecutionContext};

mod authorized;
mod responses;
mod subgraph;

pub(crate) struct RequestHooks<'ctx, H: Hooks> {
    hooks: &'ctx H,
    context: &'ctx H::Context,
}

impl<'ctx, R: Runtime> From<&PreExecutionContext<'ctx, R>> for RequestHooks<'ctx, R::Hooks> {
    fn from(ctx: &PreExecutionContext<'ctx, R>) -> Self {
        Self {
            hooks: ctx.engine.runtime.hooks(),
            context: &ctx.request_context.hooks_context,
        }
    }
}

impl<'ctx, R: Runtime> From<&ExecutionContext<'ctx, R>> for RequestHooks<'ctx, R::Hooks> {
    fn from(ctx: &ExecutionContext<'ctx, R>) -> Self {
        Self {
            hooks: ctx.engine.runtime.hooks(),
            context: &ctx.request_context.hooks_context,
        }
    }
}
