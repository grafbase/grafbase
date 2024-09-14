use futures::future::BoxFuture;
use grafbase_telemetry::metrics::EngineMetrics;
use runtime::{
    auth::AccessToken,
    hooks::{ExecutedOperation, ExecutedOperationBuilder},
};
use schema::{HeaderRule, Schema};

use crate::{
    engine::{HooksContext, RequestContext},
    Engine, Runtime,
};

use super::{header_rule::create_subgraph_headers_with_rules, ExecutableOperation, RequestHooks};

/// Context before starting the execution of the operation plan.
///
/// This struct holds the necessary components to facilitate the operation plan execution.
/// Background futures will be initiated in parallel to avoid delaying the execution process.
pub(crate) struct PreExecutionContext<'ctx, R: Runtime> {
    /// A reference to the engine managing the execution lifecycle.
    pub(crate) engine: &'ctx Engine<R>,

    /// A reference to the request context associated with the execution.
    pub(crate) request_context: &'ctx RequestContext,

    /// The context for hooks, allowing data storage for the user during the request lifecycle.
    pub(crate) hooks_context: HooksContext<R>,

    /// Builder for creating the `executed-operation` for the `on-operation-response` hook.
    pub(crate) executed_operation_builder: ExecutedOperationBuilder,

    /// A queue that holds background futures that need to be sent across threads,
    /// as they need to be `Send`.
    pub(super) background_futures: crossbeam_queue::SegQueue<BoxFuture<'ctx, ()>>,
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    /// Constructs a new instance of `PreExecutionContext`.
    ///
    /// # Parameters
    ///
    /// - `engine`: A reference to the engine managing the execution lifecycle.
    /// - `request_context`: A reference to the request context associated with the execution.
    /// - `hooks_context`: The context for hooks, allowing data storage for the user during the request lifecycle.
    ///
    /// # Returns
    ///
    /// Returns a new `PreExecutionContext` instance.
    pub fn new(engine: &'ctx Engine<R>, request_context: &'ctx RequestContext, hooks_context: HooksContext<R>) -> Self {
        Self {
            engine,
            request_context,
            hooks_context,
            executed_operation_builder: ExecutedOperation::builder(),
            background_futures: Default::default(),
        }
    }

    /// Pushes a background future onto the queue for execution.
    ///
    /// This function stores the provided future in the `background_futures` queue,
    /// allowing it to be executed in a separate thread without blocking the
    /// operation plan execution.
    ///
    /// # Parameters
    ///
    /// - `future`: A boxed future that will be executed in the background.
    pub fn push_background_future(&self, future: BoxFuture<'ctx, ()>) {
        self.background_futures.push(future)
    }

    /// Retrieves a reference to the schema associated with the engine.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `Schema` instance.
    pub fn schema(&self) -> &'ctx Schema {
        &self.engine.schema
    }

    /// Retrieves a reference to the access token associated with the request context.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `AccessToken` instance.
    pub fn access_token(&self) -> &'ctx AccessToken {
        &self.request_context.access_token
    }

    /// Retrieves a reference to the HTTP headers associated with the request context.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `http::HeaderMap` instance containing the request headers.
    pub fn headers(&self) -> &'ctx http::HeaderMap {
        &self.request_context.headers
    }

    /// Retrieves a hooks instance for the associated runtime.
    ///
    /// This method allows access to the hooks associated with the current execution context,
    /// which can be used for tracing or modifying the behavior of the execution lifecycle by the
    /// user.
    ///
    /// # Returns
    ///
    /// Returns a `RequestHooks` instance that provides access to the hooks.
    pub fn hooks(&self) -> RequestHooks<'_, R::Hooks> {
        self.into()
    }

    /// Retrieves a reference to the engine metrics associated with the runtime.
    ///
    /// # Returns
    ///
    /// Returns a reference to the `EngineMetrics` instance.
    pub fn metrics(&self) -> &'ctx EngineMetrics {
        self.engine.runtime.metrics()
    }
}

/// Data available during the executor life during its build & execution phases.
pub(crate) struct ExecutionContext<'ctx, R: Runtime> {
    /// A reference to the engine managing the execution.
    pub engine: &'ctx Engine<R>,

    /// A reference to the currently executing operation.
    pub operation: &'ctx ExecutableOperation,

    /// A reference to the request context associated with the execution.
    pub(super) request_context: &'ctx RequestContext,

    /// A reference to the hooks context, constructed in the `on-gateway-request` hook
    /// and passed into all subsequent hook calls during the request lifetime.
    pub(super) hooks_context: &'ctx HooksContext<R>,
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

    /// Generates subgraph headers based on the provided rules.
    ///
    /// # Parameters
    ///
    /// - `rules`: An iterator providing the `HeaderRule`s from configuration.
    ///
    /// # Returns
    ///
    /// Returns an `http::HeaderMap` containing the generated headers for the subgraph.
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

    /// Retrieves a reference to the schema associated with the engine.
    pub fn schema(&self) -> &'ctx Schema {
        &self.engine.schema
    }

    /// Retrieves a reference to the engine metrics associated with the runtime.
    pub fn metrics(&self) -> &'ctx EngineMetrics {
        self.engine.runtime.metrics()
    }
}
