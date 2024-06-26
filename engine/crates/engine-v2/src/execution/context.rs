use ::runtime::hooks::Hooks;
use runtime::auth::AccessToken;

use crate::{engine::RequestContext, Engine, Runtime};

use super::RequestHooks;

/// Data available during the executor life during its build & execution phases.
pub(crate) struct ExecutionContext<'ctx, R: Runtime> {
    pub engine: &'ctx Engine<R>,
    pub request_context: &'ctx RequestContext<<R::Hooks as Hooks>::Context>,
}

impl<R: Runtime> Clone for ExecutionContext<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: Runtime> std::marker::Copy for ExecutionContext<'_, R> {}

impl<'ctx, R: Runtime> std::ops::Deref for ExecutionContext<'ctx, R> {
    type Target = Engine<R>;
    fn deref(&self) -> &'ctx Self::Target {
        self.engine
    }
}

impl<'ctx, R: Runtime> ExecutionContext<'ctx, R> {
    pub fn access_token(&self) -> &'ctx AccessToken {
        &self.request_context.access_token
    }

    pub fn headers(&self) -> &'ctx http::HeaderMap {
        &self.request_context.headers
    }

    pub fn header(&self, name: &str) -> Option<&'ctx str> {
        self.headers().get(name).and_then(|v| v.to_str().ok())
    }

    pub fn hooks(&self) -> RequestHooks<'ctx, R> {
        (*self).into()
    }
}
