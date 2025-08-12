use std::sync::Arc;

use event_queue::EventQueue;
use extension_catalog::ExtensionId;
use runtime::extension::Token;

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
    pub fn event_queue(&self) -> &Arc<EventQueue> {
        &self.0.event_queue
    }

    pub fn hooks_context(&self) -> &Arc<[u8]> {
        &self.0.hooks_context
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
    pub fn event_queue(&self) -> &Arc<EventQueue> {
        &self.request.event_queue
    }

    pub fn hooks_context(&self) -> &Arc<[u8]> {
        &self.request.hooks_context
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
