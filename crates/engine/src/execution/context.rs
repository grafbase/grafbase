use std::sync::Arc;

use grafbase_telemetry::metrics::EngineMetrics;
use runtime::auth::AccessToken;
use schema::{HeaderRule, Schema};

use crate::{
    engine::{HooksContext, RequestContext},
    operation::{InputValueContext, OperationPlanContext, SolvedOperationContext, Variables},
    prepare::PreparedOperation,
    response::Shapes,
    Engine, Runtime,
};

use super::{header_rule::create_subgraph_headers_with_rules, RequestHooks};

/// Data available during the executor life during its build & execution phases.
pub(crate) struct ExecutionContext<'ctx, R: Runtime> {
    pub engine: &'ctx Arc<Engine<R>>,
    pub operation: &'ctx Arc<PreparedOperation>,
    pub request_context: &'ctx Arc<RequestContext>,
    pub hooks_context: &'ctx Arc<HooksContext<R>>,
}

impl<R: Runtime> Clone for ExecutionContext<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: Runtime> std::marker::Copy for ExecutionContext<'_, R> {}

impl<'ctx, R: Runtime> ExecutionContext<'ctx, R> {
    #[allow(unused)]
    pub fn access_token(&self) -> &'ctx AccessToken {
        &self.request_context.access_token
    }

    pub fn subgraph_headers_with_rules(&self, rules: impl Iterator<Item = HeaderRule<'ctx>>) -> http::HeaderMap {
        create_subgraph_headers_with_rules(self.request_context, rules)
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

    pub fn input_value_context(&self) -> InputValueContext<'ctx> {
        InputValueContext {
            schema: &self.engine.schema,
            query_input_values: &self.operation.cached.solved.query_input_values,
            variables: &self.operation.variables,
        }
    }

    pub fn shapes(&self) -> &'ctx Shapes {
        &self.operation.cached.solved.shapes
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
            query_input_values: &ctx.operation.cached.solved.query_input_values,
            variables: &ctx.operation.variables,
        }
    }
}

impl<'ctx, R: Runtime> From<&ExecutionContext<'ctx, R>> for SolvedOperationContext<'ctx> {
    fn from(ctx: &ExecutionContext<'ctx, R>) -> Self {
        SolvedOperationContext {
            schema: &ctx.engine.schema,
            operation: &ctx.operation.cached.solved,
        }
    }
}

impl<'ctx, R: Runtime> From<&ExecutionContext<'ctx, R>> for OperationPlanContext<'ctx> {
    fn from(ctx: &ExecutionContext<'ctx, R>) -> Self {
        OperationPlanContext {
            schema: &ctx.engine.schema,
            solved_operation: &ctx.operation.cached.solved,
            operation_plan: &ctx.operation.plan,
        }
    }
}
