use std::sync::Arc;

use futures::future::BoxFuture;
use grafbase_telemetry::metrics::EngineMetrics;
use runtime::{
    authentication::LegacyToken,
    hooks::{ExecutedOperation, ExecutedOperationBuilder, Hooks},
};
use schema::Schema;

use crate::{
    Engine, Runtime,
    engine::{WasmContext, WasmExtensionContext},
    execution::{GraphqlRequestContext, RequestContext, RequestHooks},
};

/// Context for preparing a single operation.
/// Background futures will be started in parallel with the operation execution to avoid delaying the plan,
/// if and only if operation preparation succeeds.
pub(crate) struct PrepareContext<'ctx, R: Runtime> {
    pub engine: &'ctx Arc<Engine<R>>,
    pub request_context: &'ctx Arc<RequestContext<WasmExtensionContext<R>>>,
    pub gql_context: GraphqlRequestContext<R>,
    pub executed_operation_builder: ExecutedOperationBuilder<<R::Hooks as Hooks>::OnSubgraphResponseOutput>,
    // needs to be Send so that futures are Send.
    pub background_futures: crossbeam_queue::SegQueue<BoxFuture<'ctx, ()>>,
}

impl<'ctx, R: Runtime> PrepareContext<'ctx, R> {
    pub fn new(
        engine: &'ctx Arc<Engine<R>>,
        request_context: &'ctx Arc<RequestContext<WasmExtensionContext<R>>>,
        wasm_context: WasmContext<R>,
    ) -> Self {
        Self {
            engine,
            request_context,
            gql_context: GraphqlRequestContext {
                wasm_context,
                subgraph_default_headers_override: None,
            },
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

    pub fn runtime(&self) -> &'ctx R {
        &self.engine.runtime
    }

    pub fn access_token(&self) -> &'ctx LegacyToken {
        &self.request_context.token
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

    pub fn operation_cache(&self) -> &'ctx R::OperationCache {
        self.engine.runtime.operation_cache()
    }

    pub fn extensions(&self) -> &'ctx R::Extensions {
        self.engine.runtime.extensions()
    }
}
