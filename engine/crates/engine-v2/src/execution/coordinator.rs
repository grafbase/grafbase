use std::sync::Arc;

use async_runtime::make_send_on_wasm;
use engine_parser::types::OperationType;
use futures::{stream::FuturesOrdered, Future, Stream};
use futures_util::{
    future::BoxFuture,
    stream::{BoxStream, FuturesUnordered},
    StreamExt,
};

use crate::{
    execution::{coordinator, ExecutionContext},
    plan::{ExecutionPlanId, OperationExecutionState, OperationPlan, PlanWalker},
    response::{Response, ResponseBuilder, ResponseObjectRef, ResponsePart},
    sources::{Executor, ExecutorInput, SubscriptionInput},
    Runtime,
};

use super::{ExecutionResult, PreExecutionContext};

pub(crate) trait ResponseSender {
    type Error;
    fn send(&mut self, response: Response) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    pub async fn execute_query_or_mutation(self, operation_plan: OperationPlan) -> Response {
        let background_futures: FuturesUnordered<_> = self.background_futures.into_iter().collect();
        let background_fut = background_futures.collect::<Vec<_>>();

        let coordinator = coordinator::ExecutionCoordinator::new(self.inner, operation_plan);
        let response_fut = coordinator.execute();

        let (response, _) = futures_util::join!(response_fut, background_fut);
        response
    }

    pub async fn execute_subscription(self, operation_plan: OperationPlan, responses: impl ResponseSender + Send) {
        let background_futures: FuturesUnordered<_> = self.background_futures.into_iter().collect();
        let background_fut = background_futures.collect::<Vec<_>>();

        let coordinator = coordinator::ExecutionCoordinator::new(self.inner, operation_plan);
        let subscription_fut = coordinator.execute_subscription(responses);
        futures_util::join!(subscription_fut, background_fut);
    }
}

pub(crate) struct ExecutionCoordinator<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    operation_plan: OperationPlan,
}

impl<'ctx, R: Runtime> ExecutionCoordinator<'ctx, R> {
    pub fn new(ctx: ExecutionContext<'ctx, R>, operation_plan: OperationPlan) -> Self {
        Self { ctx, operation_plan }
    }

    fn plan_walker(&self, plan_id: ExecutionPlanId) -> PlanWalker<'_, (), ()> {
        self.operation_plan.walker_with(&self.ctx.engine.schema, plan_id)
    }

    pub async fn execute(self) -> Response {
        assert!(
            !matches!(self.operation_plan.ty(), OperationType::Subscription),
            "execute shouldn't be called for subscriptions"
        );

        OperationExecution {
            coordinator: &self,
            futures: ExecutorFutureSet::new(),
            state: self.operation_plan.new_execution_state(),
            response: ResponseBuilder::new(self.operation_plan.root_object_id),
        }
        .execute()
        .await
    }

    pub async fn execute_subscription(self, mut responses: impl ResponseSender + Send) {
        assert!(matches!(self.operation_plan.ty(), OperationType::Subscription));

        if !self.operation_plan.root_errors.is_empty() {
            let mut response = ResponseBuilder::new(self.operation_plan.root_object_id);
            response.push_root_errors(&self.operation_plan.root_errors);
            let _ = responses
                .send(response.build(self.ctx.engine.schema.clone(), self.operation_plan.operation.clone()))
                .await;
            return;
        }

        let (state, subscription_plan_id) = {
            let mut state = self.operation_plan.new_execution_state();
            let id = state.pop_subscription_plan_id();
            (state, id)
        };
        let entity_locations_to_track = &self.plan_walker(subscription_plan_id).output().tracked_locations;

        let new_execution = || {
            let mut response = ResponseBuilder::new(self.operation_plan.root_object_id);
            OperationRootPlanExecution {
                root_response_part: response.new_part(
                    Arc::new(response.root_response_object().into_iter().collect()),
                    entity_locations_to_track,
                ),
                operation_execution: OperationExecution {
                    coordinator: &self,
                    futures: ExecutorFutureSet::new(),
                    state: state.clone(),
                    response,
                },
            }
        };

        let stream = match self
            .build_subscription_stream(subscription_plan_id, new_execution)
            .await
        {
            Ok(stream) => stream,
            Err(error) => Box::pin(futures_util::stream::iter(std::iter::once(Err(error)))),
        };

        SubscriptionExecution {
            subscription_plan_id,
            stream,
        }
        .execute(responses)
        .await
    }

    async fn build_subscription_stream<'s, 'caller>(
        &'s self,
        plan_id: ExecutionPlanId,
        new_execution: impl Fn() -> OperationRootPlanExecution<'caller, R> + Send + 'caller,
    ) -> ExecutionResult<BoxStream<'caller, ExecutionResult<OperationRootPlanExecution<'caller, R>>>>
    where
        's: 'caller,
    {
        let execution_plan = &self.operation_plan[plan_id];
        let plan = self.plan_walker(plan_id);
        let input = SubscriptionInput { ctx: self.ctx, plan };
        let executor = execution_plan.new_subscription_executor(input)?;
        executor.execute(new_execution).await
    }
}

struct SubscriptionExecution<S> {
    subscription_plan_id: ExecutionPlanId,
    stream: S,
}

impl<'a, R: Runtime, S> SubscriptionExecution<S>
where
    S: Stream<Item = ExecutionResult<OperationRootPlanExecution<'a, R>>> + Send,
{
    async fn execute(self, mut responses: impl ResponseSender + Send) {
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
                    let Some(execution) = execution else {
                        break;
                    };
                    match execution {
                        Ok(OperationRootPlanExecution {
                            mut operation_execution,
                            root_response_part,
                        }) => {
                            operation_execution.futures.push_result(ExecutorFutureResult {
                                result: Ok(root_response_part),
                                root_response_object_refs: Arc::new(
                                    operation_execution
                                        .response
                                        .root_response_object()
                                        .into_iter()
                                        .collect(),
                                ),
                                plan_id: self.subscription_plan_id,
                            });
                            response_futures.push_back(operation_execution.execute());
                        }
                        Err(error) => {
                            if responses.send(Response::execution_error(error)).await.is_err() {
                                return;
                            }
                        }
                    };
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

pub struct OperationRootPlanExecution<'ctx, R: Runtime> {
    operation_execution: OperationExecution<'ctx, R>,
    root_response_part: ResponsePart,
}

impl<R: Runtime> OperationRootPlanExecution<'_, R> {
    pub fn root_response_part(&mut self) -> &mut ResponsePart {
        &mut self.root_response_part
    }
}

pub struct OperationExecution<'ctx, R: Runtime> {
    coordinator: &'ctx ExecutionCoordinator<'ctx, R>,
    futures: ExecutorFutureSet<'ctx>,
    state: OperationExecutionState,
    response: ResponseBuilder,
}

impl<'ctx, R: Runtime> OperationExecution<'ctx, R> {
    /// Runs a single execution to completion, returning its response
    async fn execute(mut self) -> Response {
        if !self.coordinator.operation_plan.root_errors.is_empty() {
            self.response
                .push_root_errors(&self.coordinator.operation_plan.root_errors);
            return self.response.build(
                self.coordinator.ctx.engine.schema.clone(),
                self.coordinator.operation_plan.operation.clone(),
            );
        }
        for plan_id in self.state.get_executable_plans() {
            self.spawn_executor(plan_id);
        }

        while let Some(ExecutorFutureResult {
            plan_id,
            root_response_object_refs,
            result,
        }) = self.futures.next().await
        {
            // Retrieving the first edge (response key) appearing in the query to provide a better
            // error path if necessary.
            let selection_set = self.coordinator.plan_walker(plan_id).collected_selection_set();
            let first_edge = selection_set
                .fields()
                .map(|field| field.as_ref().edge)
                .min()
                .or_else(|| selection_set.as_ref().field_errors.first().map(|f| f.edge))
                .or_else(|| selection_set.as_ref().typename_fields.first().copied())
                .expect("Selection set without any fields?");
            let default_object = selection_set.maybe_default_object();
            match result {
                Ok(part) => {
                    tracing::trace!(%plan_id, "Succeeded");

                    for (entity_location, response_object_refs) in
                        self.response.ingest(part, first_edge, default_object)
                    {
                        self.state.push_response_objects(entity_location, response_object_refs);
                    }

                    for plan_id in self
                        .state
                        .get_next_executable_plans(&self.coordinator.operation_plan, plan_id)
                    {
                        self.spawn_executor(plan_id);
                    }
                }
                Err(error) => {
                    tracing::trace!(%plan_id, "Failed");
                    self.response.propagate_execution_error(
                        &root_response_object_refs,
                        first_edge,
                        error,
                        default_object,
                    );
                }
            };
        }

        self.response.build(
            self.coordinator.ctx.engine.schema.clone(),
            self.coordinator.operation_plan.operation.clone(),
        )
    }

    fn spawn_executor(&mut self, plan_id: ExecutionPlanId) {
        tracing::trace!(%plan_id, "Starting plan");
        let operation: &'ctx OperationPlan = &self.coordinator.operation_plan;
        let engine = self.coordinator.ctx.engine;
        let root_response_object_refs =
            self.state
                .get_root_response_object_refs(&engine.schema, operation, &self.response, plan_id);

        tracing::trace!(%plan_id, "Found {} root response objects", root_response_object_refs.len());
        if root_response_object_refs.is_empty() {
            return;
        }

        let execution_plan = &operation[plan_id];
        let plan = self.coordinator.plan_walker(plan_id);
        let response_part = self
            .response
            .new_part(root_response_object_refs.clone(), &plan.output().tracked_locations);
        let input = ExecutorInput {
            ctx: self.coordinator.ctx,
            plan,
            root_response_objects: self.response.read(
                plan.schema(),
                root_response_object_refs.clone(),
                &plan.input().selection_set,
            ),
        };

        match execution_plan.new_executor(input) {
            Ok(executor) => self
                .futures
                .execute(plan_id, root_response_object_refs, executor, response_part),
            Err(error) => {
                self.futures.push_result(ExecutorFutureResult {
                    result: Err(error),
                    root_response_object_refs,
                    plan_id,
                });
            }
        }
    }
}

pub struct ExecutorFutureSet<'a>(FuturesUnordered<BoxFuture<'a, ExecutorFutureResult>>);

impl<'a> ExecutorFutureSet<'a> {
    fn new() -> Self {
        ExecutorFutureSet(FuturesUnordered::new())
    }

    fn execute<R: Runtime>(
        &mut self,
        plan_id: ExecutionPlanId,
        root_response_object_refs: Arc<Vec<ResponseObjectRef>>,
        executor: Executor<'a, R>,
        response_part: ResponsePart,
    ) {
        self.0.push(Box::pin(make_send_on_wasm(async move {
            let result = executor.execute(response_part).await;
            ExecutorFutureResult {
                plan_id,
                root_response_object_refs,
                result,
            }
        })));
    }

    fn push_result(&mut self, result: ExecutorFutureResult) {
        self.0.push(Box::pin(async move { result }));
    }

    async fn next(&mut self) -> Option<ExecutorFutureResult> {
        self.0.next().await
    }
}

struct ExecutorFutureResult {
    plan_id: ExecutionPlanId,
    root_response_object_refs: Arc<Vec<ResponseObjectRef>>,
    result: ExecutionResult<ResponsePart>,
}
