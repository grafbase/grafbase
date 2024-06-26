use std::sync::Arc;

use crate::response::GraphqlError;

use super::ExecutionContext;

pub(crate) struct RequestHooks<'ctx>(ExecutionContext<'ctx>);

impl<'ctx> From<ExecutionContext<'ctx>> for RequestHooks<'ctx> {
    fn from(ctx: ExecutionContext<'ctx>) -> Self {
        Self(ctx)
    }
}

impl<'ctx> RequestHooks<'ctx> {
    pub async fn authorized(&self, rule: String) -> Option<GraphqlError> {
        let results = self
            .0
            .engine
            .env
            .hooks
            .authorized(Arc::clone(&self.0.request_metadata.context), rule, vec![String::new()])
            .await;
        tracing::info!("{results:#?}");
        match results {
            Ok(authorization_errors) => authorization_errors
                .into_iter()
                .next()
                .map(|maybe_error| maybe_error.map(Into::into))
                .unwrap_or_else(|| Some(GraphqlError::internal_server_error())),
            Err(err) => {
                if !err.is_user_error() {
                    tracing::error!("Hook error: {err:?}");
                }
                Some(err.into())
            }
        }
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
