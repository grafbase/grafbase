use std::{collections::VecDeque, sync::Arc};

use event_queue::{ExecutedOperation, ExecutedOperationBuilder};
use futures::{Future, FutureExt, Stream, stream::FuturesOrdered};
use futures_util::{StreamExt, future::BoxFuture, stream::FuturesUnordered};
use grafbase_telemetry::graphql::{GraphqlResponseStatus, OperationType};
use tracing::Instrument;
use walker::Walk;

use crate::{
    Runtime,
    execution::ExecutionContext,
    prepare::{Executable, Plan, PlanId, PrepareContext, PreparedOperation},
    resolver::ResolverResult,
    response::{GraphqlError, PartIngestionResult, Response, ResponseBuilder, ResponsePartBuilder},
};

use super::state::OperationExecutionState;

pub(crate) trait ResponseSender: Send {
    type Error;
    fn send(&mut self, response: Response) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

impl<R: Runtime> PrepareContext<'_, R> {
    pub async fn execute_query_or_mutation(mut self, operation: PreparedOperation) -> Response {
        let background_futures: FuturesUnordered<_> =
            std::mem::take(&mut self.background_futures).into_iter().collect();

        let background_fut = background_futures.collect::<Vec<_>>();
        let operation = Arc::new(operation);

        let ctx = ExecutionContext {
            engine: self.engine,
            request_context: self.request_context,
            operation: &operation,
            gql_context: &self.gql_context,
        };

        tracing::trace!("Starting execution...");

        if operation.plan.query_modifications.root_error_ids.is_empty() {
            let response_fut = ctx.execute(self.executed_operation_builder);
            let (response, _) = futures_util::join!(response_fut, background_fut);

            response
        } else {
            let response_fut = ctx.response_for_root_errors(self.executed_operation_builder);
            let (response, _) = futures_util::join!(response_fut, background_fut);

            response
        }
    }

    pub async fn execute_subscription(mut self, operation: PreparedOperation, mut responses: impl ResponseSender) {
        let background_futures: FuturesUnordered<_> =
            std::mem::take(&mut self.background_futures).into_iter().collect();

        let background_fut = background_futures.collect::<Vec<_>>();
        let operation = Arc::new(operation);

        let ctx = ExecutionContext {
            engine: self.engine,
            request_context: self.request_context,
            operation: &operation,
            gql_context: &self.gql_context,
        };

        tracing::trace!("Starting execution...");

        if operation.plan.query_modifications.root_error_ids.is_empty() {
            let subscription_fut = ctx.execute_subscription(self.executed_operation_builder, responses);
            futures_util::join!(subscription_fut, background_fut);
        } else {
            let response_fut = ctx.response_for_root_errors(self.executed_operation_builder);
            let (response, _) = futures_util::join!(response_fut, background_fut);

            responses.send(response).await.ok();
        }
    }
}

impl<'ctx, R: Runtime> ExecutionContext<'ctx, R> {
    fn new_execution_state(&self) -> OperationExecutionState<'ctx, R> {
        OperationExecutionState::new(*self)
    }

    fn execution_error(&self, errors: impl IntoIterator<Item: Into<GraphqlError>>) -> Response {
        Response::execution_error(&self.engine.schema, self.operation, errors)
    }

    async fn response_for_root_errors(self, mut builder: ExecutedOperationBuilder<'_>) -> Response {
        builder.status(GraphqlResponseStatus::FieldError {
            count: self.operation.plan.query_modifications.root_error_ids.len() as u64,
            data_is_null: true,
        });

        if let Some(name) = self.operation.cached.operation.attributes.name.original() {
            builder.name(name);
        }

        self.event_queue().push_operation(builder);

        self.execution_error(
            self.operation
                .plan
                .query_modifications
                .root_error_ids
                .iter()
                .copied()
                .map(|id| self.operation.plan.query_modifications[id].clone()),
        )
    }

    async fn execute(self, builder: ExecutedOperationBuilder<'_>) -> Response {
        assert!(
            !matches!(self.operation.cached.ty(), OperationType::Subscription),
            "execute shouldn't be called for subscriptions"
        );

        OperationExecution {
            state: self.new_execution_state(),
            executed_operation_builder: builder,
            response: ResponseBuilder::new(&self.engine.schema, self.operation),
            ctx: self,
        }
        .run(VecDeque::new())
        .await
    }

    async fn execute_subscription(
        self,
        executed_operation_builder: ExecutedOperationBuilder<'_>,
        responses: impl ResponseSender,
    ) {
        assert!(matches!(self.operation.cached.ty(), OperationType::Subscription));

        let (initial_state, subscription_plan) = {
            let mut state = self.new_execution_state();
            let id = state.pop_subscription_plan();
            (state, id)
        };

        let stream = subscription_plan
            .as_ref()
            .resolver
            .execute_subscription(self, subscription_plan, move || {
                ResponseBuilder::new(&self.engine.schema, self.operation)
            })
            .await;

        SubscriptionExecution {
            ctx: self,
            subscription_plan,
            first_executed_operation_builder: Some(executed_operation_builder),
            initial_state,
            stream,
        }
        .execute(responses)
        .await
    }
}

struct SubscriptionExecution<'ctx, R: Runtime, S> {
    ctx: ExecutionContext<'ctx, R>,
    subscription_plan: Plan<'ctx>,
    initial_state: OperationExecutionState<'ctx, R>,
    first_executed_operation_builder: Option<ExecutedOperationBuilder<'ctx>>,
    stream: S,
}

impl<'ctx, R: Runtime, S> SubscriptionExecution<'ctx, R, S>
where
    S: Stream<Item = (ResponseBuilder<'ctx>, ResponsePartBuilder<'ctx>)> + Send,
{
    async fn execute(mut self, mut responses: impl ResponseSender) {
        let subscription_stream = self.stream.fuse();
        futures_util::pin_mut!(subscription_stream);

        let mut response_futures = FuturesOrdered::new();
        loop {
            let next_task = if response_futures.is_empty() {
                Err(subscription_stream.next().await)
            } else {
                // If we have already enough ongoing futures we don't continue to poll the stream
                // to apply some back pressure.
                if response_futures.len() < 3 {
                    // We try to finish ongoing responses first, but while waiting we continue
                    // polling the stream for the next one.
                    futures_util::select_biased! {
                        response = response_futures.next() => Ok(response),
                        execution = subscription_stream.next() => Err(execution),
                    }
                } else {
                    Ok(response_futures.next().await)
                }
            };

            match next_task {
                Ok(response) => {
                    // Should never be None as we only wait for the futures if there is something
                    // to wait for.
                    if let Some(response) = response {
                        if responses.send(response).await.is_err() {
                            return;
                        }
                    }
                }
                Err(execution) => {
                    let Some((response, response_part)) = execution else {
                        break;
                    };

                    let executed_operation_builder = self
                        .first_executed_operation_builder
                        .take()
                        .unwrap_or_else(|| ExecutedOperation::builder(event_queue::OperationType::Subscription));

                    let mut results = VecDeque::new();
                    results.push_back(PlanExecutionResult {
                        plan_id: self.subscription_plan.id,
                        response_part,
                    });

                    let operation_execution = OperationExecution {
                        ctx: self.ctx,
                        executed_operation_builder,
                        state: self.initial_state.clone(),
                        response,
                    };

                    response_futures.push_back(operation_execution.run(results));
                }
            }
        }
        // Finishing any remaining responses after the subscription stream ended.
        while let Some(response) = response_futures.next().await {
            if responses.send(response).await.is_err() {
                return;
            }
        }
    }
}

struct OperationExecution<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    executed_operation_builder: ExecutedOperationBuilder<'ctx>,
    state: OperationExecutionState<'ctx, R>,
    response: ResponseBuilder<'ctx>,
}

impl<'ctx, R: Runtime> std::ops::Deref for OperationExecution<'ctx, R> {
    type Target = ExecutionContext<'ctx, R>;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

enum State<F, S> {
    Ingestion(F),
    Execution(S),
}

enum TaskResult<U, V> {
    Ingestion(U),
    Execution(V),
}

impl<'ctx, R: Runtime> OperationExecution<'ctx, R> {
    /// Runs a single execution to completion, returning its response
    async fn run(mut self, mut results: VecDeque<PlanExecutionResult<'ctx>>) -> Response {
        let futures = FuturesUnordered::new();
        let initial_plans = self.state.get_executable_plans().collect::<Vec<_>>();
        let event_queue = self.ctx.event_queue();

        for plan in initial_plans {
            if let Some(fut) = self.create_plan_execution_future(plan) {
                futures.push(fut);
            }
        }

        let mut state = State::Execution(self);
        futures_util::pin_mut!(futures);

        let mut this = loop {
            state = match state {
                State::Ingestion(mut ingestion_fut) => {
                    let task_result = futures_util::select_biased! {
                        ingestion_result = ingestion_fut => TaskResult::Ingestion(ingestion_result),
                        execution_result = futures.next() => TaskResult::Execution(execution_result),
                    };

                    match task_result {
                        TaskResult::Ingestion((this, next_futures)) => {
                            for fut in next_futures {
                                futures.push(fut);
                            }
                            State::Execution(this)
                        }
                        TaskResult::Execution(Some(result)) => {
                            results.push_back(result);
                            State::Ingestion(ingestion_fut)
                        }
                        TaskResult::Execution(None) => {
                            let (self_, next_futures) = ingestion_fut.await;
                            for fut in next_futures {
                                futures.push(fut)
                            }
                            State::Execution(self_)
                        }
                    }
                }
                State::Execution(this) => {
                    if let Some(result) = results.pop_front() {
                        State::Ingestion(Box::pin(this.ingest_execution_result(result).fuse()))
                    } else if let Some(result) = futures.next().await {
                        results.push_back(result);
                        State::Execution(this)
                    } else {
                        break this;
                    }
                }
            };
        };

        let operation = this.ctx.operation;

        if let Some(name) = operation.cached.operation.attributes.name.original() {
            this.executed_operation_builder.name(name);
        }

        this.executed_operation_builder
            .document(&operation.cached.operation.attributes.sanitized_query)
            .status(this.response.graphql_status());

        if let Some(complexity) = operation.complexity_cost {
            this.executed_operation_builder.complexity(complexity.0 as u64);
        }

        event_queue.push_operation(this.executed_operation_builder);

        this.response.build(operation.attributes())
    }

    async fn ingest_execution_result<'exec>(
        mut self,
        PlanExecutionResult { plan_id, response_part }: PlanExecutionResult<'ctx>,
    ) -> (Self, Vec<BoxFuture<'exec, PlanExecutionResult<'ctx>>>)
    where
        'ctx: 'exec,
    {
        let PartIngestionResult::Data { response_object_sets } = self.response.ingest(response_part) else {
            tracing::trace!(%plan_id, "Failed");
            return (self, Vec::new());
        };

        tracing::trace!(%plan_id, "Succeeded");

        for (set_id, response_object_refs) in response_object_sets {
            self.state.push_response_objects(set_id, response_object_refs);
        }

        let plan = plan_id.walk(&self.ctx);
        let mut stack = self.state.get_next_executables(plan);
        let mut next_futures = Vec::new();

        while let Some(executable) = stack.pop() {
            tracing::trace!("Running {:?}", executable.id());

            match executable {
                Executable::Plan(plan) => {
                    if let Some(fut) = self.create_plan_execution_future(plan) {
                        next_futures.push(fut);
                    }
                }
                Executable::ResponseModifier(response_modifier) => {
                    self.ctx
                        .execute_response_modifier(&mut self.state, &mut self.response, response_modifier)
                        .await;
                    stack.append(&mut self.state.get_next_executables(response_modifier));
                }
            }
        }

        (self, next_futures)
    }

    fn create_plan_execution_future<'exec>(
        &mut self,
        plan: Plan<'ctx>,
    ) -> Option<BoxFuture<'exec, PlanExecutionResult<'ctx>>>
    where
        'ctx: 'exec,
    {
        tracing::trace!(plan_id = %plan.id, "Starting plan");
        let parent_objects = self.state.get_input(&self.response, plan);

        tracing::trace!(
            plan_id = %plan.id,
            "Found {} root response objects",
            parent_objects.len()
        );

        if parent_objects.is_empty() {
            return None;
        }

        let span = tracing::debug_span!("resolver", "plan_id" = usize::from(plan.id)).entered();

        let response_part = self.response.create_part();
        let parent_objects_view = self.response.read(parent_objects, plan.required_fields());

        let fut = plan
            .as_ref()
            .resolver
            .execute(self.ctx, plan, parent_objects_view, response_part)
            .map(move |ResolverResult { response_part }| PlanExecutionResult {
                plan_id: plan.id,
                response_part,
            });

        let span = span.exit();
        Some(fut.instrument(span).boxed())
    }
}

pub(crate) struct PlanExecutionResult<'ctx> {
    plan_id: PlanId,
    response_part: ResponsePartBuilder<'ctx>,
}
