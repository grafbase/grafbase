use std::sync::Arc;

use extension_catalog::ExtensionId;
use runtime::extension::{ExtensionRequestContext, TokenRef};

use crate::{
    Runtime,
    execution::{ExecutionContext, RequestContext},
    prepare::PreparedOperation,
};

#[derive(Clone)]
pub struct EngineRequestContext(Arc<RequestContext>);

impl From<&Arc<RequestContext>> for EngineRequestContext {
    fn from(ctx: &Arc<RequestContext>) -> Self {
        Self(ctx.clone())
    }
}

impl EngineRequestContext {
    pub fn extension(&self) -> &Arc<ExtensionRequestContext> {
        &self.0.extension
    }

    pub fn token(&self) -> TokenRef<'_> {
        self.0.token.as_ref()
    }
}

#[derive(Clone)]
pub struct EngineOperationContext {
    request: Arc<RequestContext>,
    operation: Arc<PreparedOperation>,
}

impl<R: Runtime> From<&ExecutionContext<'_, R>> for EngineOperationContext {
    fn from(ctx: &ExecutionContext<'_, R>) -> Self {
        Self {
            request: ctx.request_context.clone(),
            operation: ctx.operation.clone(),
        }
    }
}

impl EngineOperationContext {
    pub fn extension(&self) -> &Arc<ExtensionRequestContext> {
        &self.request.extension
    }

    pub fn token(&self) -> TokenRef<'_> {
        self.request.token.as_ref()
    }

    pub fn authorization_context(&self) -> &[(ExtensionId, Vec<u8>)] {
        &self.operation.plan.query_modifications.extension.authorization_context
    }

    pub fn authorization_state(&self) -> &[(ExtensionId, Vec<u8>)] {
        &self.operation.plan.query_modifications.extension.authorization_state
    }
}
