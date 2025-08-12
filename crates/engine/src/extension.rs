use std::sync::Arc;

use extension_catalog::ExtensionId;
use runtime::extension::{ExtensionRequestContext, Token};

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
    pub fn extension(&self) -> &ExtensionRequestContext {
        &self.0.extension
    }

    pub fn token(&self) -> &Token {
        &self.0.token
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
    pub fn extension(&self) -> &ExtensionRequestContext {
        &self.request.extension
    }

    pub fn token(&self) -> &Token {
        &self.request.token
    }

    pub fn authorization_context(&self) -> &[(ExtensionId, Arc<[u8]>)] {
        &self.operation.plan.query_modifications.extension.authorization_context
    }

    pub fn authorization_state(&self) -> &[(ExtensionId, Vec<u8>)] {
        &self.operation.plan.query_modifications.extension.authorization_state
    }
}
