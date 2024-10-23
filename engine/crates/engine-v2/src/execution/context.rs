use std::sync::Arc;

use futures::future::BoxFuture;
use grafbase_telemetry::metrics::EngineMetrics;
use runtime::{
    auth::AccessToken,
    hooks::{ExecutedOperation, ExecutedOperationBuilder, Hooks},
};
use schema::{HeaderRule, Schema};

use crate::{
    engine::{HooksContext, RequestContext},
    Engine, Runtime,
};

use super::{header_rule::create_subgraph_headers_with_rules, ExecutableOperation, RequestHooks};

/// Context before starting to operation plan execution.
/// Background futures will be started in parallel to avoid delaying the plan.
pub(crate) struct PreExecutionContext<'ctx, R: Runtime> {
    pub(crate) engine: &'ctx Arc<Engine<R>>,
    pub(crate) request_context: &'ctx Arc<RequestContext>,
    pub(crate) hooks_context: HooksContext<R>,
    pub(crate) executed_operation_builder: ExecutedOperationBuilder<<R::Hooks as Hooks>::OnSubgraphResponseOutput>,
    // needs to be Send so that futures are Send.
    pub(super) background_futures: crossbeam_queue::SegQueue<BoxFuture<'ctx, ()>>,
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    pub fn new(
        engine: &'ctx Arc<Engine<R>>,
        request_context: &'ctx Arc<RequestContext>,
        hooks_context: HooksContext<R>,
    ) -> Self {
        Self {
            engine,
            request_context,
            hooks_context,
            executed_operation_builder: ExecutedOperation::builder(),
            background_futures: Default::default(),
        }
    }

    pub fn push_background_future(&self, future: BoxFuture<'ctx, ()>) {
        self.background_futures.push(future)
    }

    pub fn schema(&self) -> &'ctx Schema {
        &self.engine.schema
    }

    pub fn access_token(&self) -> &'ctx AccessToken {
        &self.request_context.access_token
    }

    pub fn headers(&self) -> &'ctx http::HeaderMap {
        &self.request_context.headers
    }

    pub fn hooks(&self) -> RequestHooks<'_, R::Hooks> {
        self.into()
    }

    pub fn metrics(&self) -> &'ctx EngineMetrics {
        self.engine.runtime.metrics()
    }
}

/// Data available during the executor life during its build & execution phases.
pub(crate) struct ExecutionContext<'ctx, R: Runtime> {
    pub engine: &'ctx Arc<Engine<R>>,
    pub operation: &'ctx Arc<ExecutableOperation>,
    pub request_context: &'ctx Arc<RequestContext>,
    pub hooks_context: &'ctx Arc<HooksContext<R>>,
}

impl<R: Runtime> Clone for ExecutionContext<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

pub(crate) struct StaticContext<R: Runtime> {
    pub engine: Arc<Engine<R>>,
    pub operation: Arc<ExecutableOperation>,
    pub request_context: Arc<RequestContext>,
    pub hooks_context: Arc<HooksContext<R>>,
}

impl<R: Runtime> std::marker::Copy for ExecutionContext<'_, R> {}

impl<'ctx, R: Runtime> ExecutionContext<'ctx, R> {
    pub fn into_static(self) -> StaticContext<R> {
        StaticContext {
            engine: Arc::clone(self.engine),
            operation: Arc::clone(self.operation),
            request_context: Arc::clone(self.request_context),
            hooks_context: Arc::clone(self.hooks_context),
        }
    }

    pub fn from_static(parts: &'ctx StaticContext<R>) -> Self {
        let StaticContext {
            engine,
            operation,
            request_context,
            hooks_context,
        } = parts;
        Self {
            engine,
            operation,
            request_context,
            hooks_context,
        }
    }

    #[allow(unused)]
    pub fn access_token(&self) -> &'ctx AccessToken {
        &self.request_context.access_token
    }

    pub fn subgraph_headers_with_rules(&self, rules: impl Iterator<Item = HeaderRule<'ctx>>) -> http::HeaderMap {
        create_subgraph_headers_with_rules(
            self.request_context,
            rules,
            self.operation.subgraph_default_headers.clone(),
        )
    }

    #[allow(unused)]
    pub fn hooks(&self) -> RequestHooks<'ctx, R::Hooks> {
        self.into()
    }

    pub fn schema(&self) -> &'ctx Schema {
        &self.engine.schema
    }

    pub fn metrics(&self) -> &'ctx EngineMetrics {
        self.engine.runtime.metrics()
    }
}
