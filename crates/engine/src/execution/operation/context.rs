use std::sync::Arc;

use event_queue::EventQueue;
use grafbase_telemetry::metrics::EngineMetrics;
use operation::{InputValueContext, Variables};
use runtime::extension::ExtensionContext as _;
use schema::{HeaderRule, Schema};

use crate::{
    Engine, Runtime,
    engine::ExtensionContext,
    execution::{GraphqlRequestContext, RequestContext, apply_header_rules},
    prepare::{CachedOperationContext, OperationPlanContext, PreparedOperation, Shapes},
};

/// Context for a single prepared operation that only needs to be executed.
pub(crate) struct ExecutionContext<'ctx, R: Runtime> {
    pub engine: &'ctx Arc<Engine<R>>,
    pub request_context: &'ctx Arc<RequestContext<ExtensionContext<R>>>,
    pub operation: &'ctx Arc<PreparedOperation>,
    pub gql_context: &'ctx GraphqlRequestContext<R>,
}

impl<R: Runtime> Clone for ExecutionContext<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: Runtime> std::marker::Copy for ExecutionContext<'_, R> {}

impl<'ctx, R: Runtime> ExecutionContext<'ctx, R> {
    pub fn event_queue(&self) -> &'ctx EventQueue {
        self.request_context.extension_context.event_queue()
    }

    pub fn subgraph_headers_with_rules(&self, rules: impl Iterator<Item = HeaderRule<'ctx>>) -> http::HeaderMap {
        let mut subgraph_headers = self
            .gql_context
            .subgraph_default_headers_override
            .as_ref()
            .unwrap_or(&self.request_context.subgraph_default_headers)
            .clone();
        apply_header_rules(&self.request_context.headers, rules, &mut subgraph_headers);
        subgraph_headers
    }

    pub fn extensions(&self) -> &'ctx R::Extensions {
        self.engine.runtime.extensions()
    }

    pub fn schema(&self) -> &'ctx Schema {
        &self.engine.schema
    }

    pub fn runtime(&self) -> &'ctx R {
        &self.engine.runtime
    }

    pub fn variables(&self) -> &'ctx Variables {
        &self.operation.variables
    }

    pub fn metrics(&self) -> &'ctx EngineMetrics {
        self.engine.runtime.metrics()
    }

    pub fn input_value_context(&self) -> InputValueContext<'ctx> {
        InputValueContext {
            schema: &self.engine.schema,
            query_input_values: &self.operation.cached.operation.query_input_values,
            variables: &self.operation.variables,
        }
    }

    pub fn shapes(&self) -> &'ctx Shapes {
        &self.operation.cached.shapes
    }
}

impl<'ctx, R: Runtime> From<&ExecutionContext<'ctx, R>> for &'ctx Variables {
    fn from(ctx: &ExecutionContext<'ctx, R>) -> Self {
        &ctx.operation.variables
    }
}

impl<'ctx, R: Runtime> From<&ExecutionContext<'ctx, R>> for &'ctx Schema {
    fn from(ctx: &ExecutionContext<'ctx, R>) -> Self {
        &ctx.engine.schema
    }
}

impl<'ctx, R: Runtime> From<&ExecutionContext<'ctx, R>> for InputValueContext<'ctx> {
    fn from(ctx: &ExecutionContext<'ctx, R>) -> Self {
        InputValueContext {
            schema: &ctx.engine.schema,
            query_input_values: &ctx.operation.cached.operation.query_input_values,
            variables: &ctx.operation.variables,
        }
    }
}

impl<'ctx, R: Runtime> From<&ExecutionContext<'ctx, R>> for CachedOperationContext<'ctx> {
    fn from(ctx: &ExecutionContext<'ctx, R>) -> Self {
        CachedOperationContext {
            schema: &ctx.engine.schema,
            cached: &ctx.operation.cached,
        }
    }
}

impl<'ctx, R: Runtime> From<&ExecutionContext<'ctx, R>> for OperationPlanContext<'ctx> {
    fn from(ctx: &ExecutionContext<'ctx, R>) -> Self {
        OperationPlanContext {
            schema: &ctx.engine.schema,
            cached: &ctx.operation.cached,
            plan: &ctx.operation.plan,
        }
    }
}
