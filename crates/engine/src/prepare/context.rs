use std::sync::Arc;

use event_queue::{ExecutedOperation, ExecutedOperationBuilder};
use futures::future::BoxFuture;
use grafbase_telemetry::metrics::EngineMetrics;
use runtime::extension::Token;
use schema::Schema;

use crate::{Engine, Runtime, execution::RequestContext};

/// Context for preparing a single operation.
/// Background futures will be started in parallel with the operation execution to avoid delaying the plan,
/// if and only if operation preparation succeeds.
pub(crate) struct PrepareContext<'ctx, R: Runtime> {
    pub engine: &'ctx Arc<Engine<R>>,
    pub request_context: &'ctx Arc<RequestContext>,
    pub executed_operation_builder: ExecutedOperationBuilder<'ctx>,
    // needs to be Send so that futures are Send.
    pub background_futures: crossbeam_queue::SegQueue<BoxFuture<'ctx, ()>>,
}

impl<'ctx, R: Runtime> PrepareContext<'ctx, R> {
    pub fn new(engine: &'ctx Arc<Engine<R>>, request_context: &'ctx Arc<RequestContext>) -> Self {
        Self {
            engine,
            request_context,
            executed_operation_builder: ExecutedOperation::builder_with_default(),
            background_futures: Default::default(),
        }
    }

    pub fn push_background_future(&self, future: BoxFuture<'ctx, ()>) {
        self.background_futures.push(future)
    }

    pub fn schema(&self) -> &'ctx Schema {
        &self.engine.schema
    }

    pub fn runtime(&self) -> &'ctx R {
        &self.engine.runtime
    }

    pub fn access_token(&self) -> &'ctx Token {
        &self.request_context.token
    }

    pub fn headers(&self) -> &'ctx http::HeaderMap {
        &self.request_context.headers
    }

    pub fn metrics(&self) -> &'ctx EngineMetrics {
        self.engine.runtime.metrics()
    }

    pub fn operation_cache(&self) -> &'ctx R::OperationCache {
        self.engine.runtime.operation_cache()
    }

    pub fn extensions(&self) -> &'ctx R::Extensions {
        self.engine.runtime.extensions()
    }
}
