use std::sync::Arc;

use extension_catalog::ExtensionId;
use runtime::extension::TokenRef;

use crate::{
    Runtime,
    execution::{ExecutionContext, RequestContext},
    prepare::PreparedOperation,
};

pub struct EngineAuthenticatedContext(Arc<RequestContext>);

impl From<&Arc<RequestContext>> for EngineAuthenticatedContext {
    fn from(ctx: &Arc<RequestContext>) -> Self {
        Self(ctx.clone())
    }
}

impl runtime::extension::OnRequestContext for EngineAuthenticatedContext {
    fn event_queue(&self) -> &event_queue::EventQueue {
        &self.0.extension.event_queue
    }

    fn hooks_context(&self) -> &[u8] {
        &self.0.extension.hooks_context
    }
}

impl runtime::extension::AuthenticatedContext for EngineAuthenticatedContext {
    fn token(&self) -> TokenRef<'_> {
        self.0.token.as_ref()
    }
}

pub struct EngineAuthorizedContext {
    request: Arc<RequestContext>,
    operation: Arc<PreparedOperation>,
}

impl<R: Runtime> From<&ExecutionContext<'_, R>> for EngineAuthorizedContext {
    fn from(ctx: &ExecutionContext<'_, R>) -> Self {
        Self {
            request: ctx.request_context.clone(),
            operation: ctx.operation.clone(),
        }
    }
}

impl runtime::extension::OnRequestContext for EngineAuthorizedContext {
    fn event_queue(&self) -> &event_queue::EventQueue {
        &self.request.extension.event_queue
    }

    fn hooks_context(&self) -> &[u8] {
        &self.request.extension.hooks_context
    }
}

impl runtime::extension::AuthenticatedContext for EngineAuthorizedContext {
    fn token(&self) -> TokenRef<'_> {
        self.request.token.as_ref()
    }
}

impl runtime::extension::AuthorizedContext for EngineAuthorizedContext {
    fn authorization_context(&self) -> &[(ExtensionId, Vec<u8>)] {
        &self.operation.plan.query_modifications.extension.authorization_context
    }

    fn authorization_state(&self) -> &[(ExtensionId, Vec<u8>)] {
        &self.operation.plan.query_modifications.extension.authorization_state
    }
}
