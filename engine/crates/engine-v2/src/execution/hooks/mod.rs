use crate::response::GraphqlError;

use super::ExecutionContext;

pub(crate) mod authorized;

pub(crate) struct RequestHooks<'ctx>(ExecutionContext<'ctx>);

impl<'ctx> From<ExecutionContext<'ctx>> for RequestHooks<'ctx> {
    fn from(ctx: ExecutionContext<'ctx>) -> Self {
        Self(ctx)
    }
}

impl From<runtime::hooks::HookError> for GraphqlError {
    fn from(err: runtime::hooks::HookError) -> Self {
        match err {
            runtime::hooks::HookError::User(err) => err.into(),
            runtime::hooks::HookError::Internal(_) => GraphqlError::internal_server_error(),
        }
    }
}

impl From<runtime::hooks::UserError> for GraphqlError {
    fn from(err: runtime::hooks::UserError) -> Self {
        GraphqlError {
            message: err.message,
            extensions: err.extensions,
            ..Default::default()
        }
    }
}
