use crate::Runtime;

use super::ExecutionContext;

pub(crate) mod authorized;

pub(crate) struct RequestHooks<'ctx, R: Runtime>(ExecutionContext<'ctx, R>);

impl<'ctx, R: Runtime> From<ExecutionContext<'ctx, R>> for RequestHooks<'ctx, R> {
    fn from(ctx: ExecutionContext<'ctx, R>) -> Self {
        Self(ctx)
    }
}
