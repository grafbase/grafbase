use ::runtime::hooks::Hooks;
use futures::future::BoxFuture;
use runtime::auth::AccessToken;

use crate::{engine::RequestContext, Engine, Runtime};

use super::RequestHooks;

/// Context before starting to operation plan execution.
/// Background futures will be started in parallel to avoid delaying the plan.
pub(crate) struct PreExecutionContext<'ctx, R: Runtime> {
    pub(super) inner: ExecutionContext<'ctx, R>,
    pub(super) background_futures: crossbeam_queue::SegQueue<BoxFuture<'ctx, ()>>,
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    pub fn new(engine: &'ctx Engine<R>, request_context: &'ctx RequestContext<<R::Hooks as Hooks>::Context>) -> Self {
        Self {
            inner: ExecutionContext {
                engine,
                request_context,
            },
            background_futures: Default::default(),
        }
    }

    pub fn push_background_future(&mut self, future: BoxFuture<'ctx, ()>) {
        self.background_futures.push(future)
    }
}

impl<'ctx, R: Runtime> std::ops::Deref for PreExecutionContext<'ctx, R> {
    type Target = ExecutionContext<'ctx, R>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

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
