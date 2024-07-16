use std::sync::Arc;

use async_runtime::make_send_on_wasm;
use engine_parser::types::OperationType;
use futures::{stream::FuturesOrdered, Future, Stream};
use futures_util::{
    future::BoxFuture,
    stream::{BoxStream, FuturesUnordered},
    StreamExt,
};
use tracing::instrument;

use crate::{
    execution::{ExecutableOperation, ExecutionContext, PlanWalker},
    response::{
        ObjectIdentifier, Response, ResponseBuilder, ResponseEdge, ResponseObjectRef, ResponsePart, ResponseValue,
    },
    sources::{Executor, ExecutorInput, SubscriptionInput},
    Runtime,
};

use super::{state::OperationExecutionState, ExecutionPlanId, ExecutionResult, PreExecutionContext};

pub(crate) trait ResponseSender: Send {
    type Error;
    fn send(&mut self, response: Response) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    #[instrument(skip_all)]
    pub async fn execute_query_or_mutation(self, operation: ExecutableOperation) -> Response {
        let background_futures: FuturesUnordered<_> = self.background_futures.into_iter().collect();
        let background_fut = background_futures.collect::<Vec<_>>();
        let ctx = ExecutionContext {
            engine: self.engine,
            operation: &operation,
            request_context: self.request_context,
        };

        let response_fut = ExecutionCoordinator { ctx }.execute();

        tracing::trace!("Starting execution...");
        let (response, _) = futures_util::join!(response_fut, background_fut);
        response
    }

    #[instrument(skip_all)]
    pub async fn execute_subscription(self, operation: ExecutableOperation, responses: impl ResponseSender) {
        let background_futures: FuturesUnordered<_> = self.background_futures.into_iter().collect();
        let background_fut = background_futures.collect::<Vec<_>>();
        let ctx = ExecutionContext {
            engine: self.engine,
            operation: &operation,
            request_context: self.request_context,
        };

        let subscription_fut = ExecutionCoordinator { ctx }.execute_subscription(responses);

        tracing::trace!("Starting execution...");
        futures_util::join!(subscription_fut, background_fut);
    }
}

pub(crate) struct ExecutionCoordinator<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
}

impl<'ctx, R: Runtime> std::ops::Deref for ExecutionCoordinator<'ctx, R> {
    type Target = ExecutionContext<'ctx, R>;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<'ctx, R: Runtime> ExecutionCoordinator<'ctx, R> {
    fn new_execution_state(&self) -> OperationExecutionState<'ctx> {
        OperationExecutionState::new(&self.engine.schema, self.operation)
    }

    fn plan_walker(&self, plan_id: ExecutionPlanId) -> PlanWalker<'ctx, (), ()> {
        PlanWalker {
            schema_walker: self.engine.schema.walker(),
            operation: self.operation,
            plan_id,
            item: (),
        }
    }

    pub async fn execute(self) -> Response {
        assert!(
            !matches!(self.operation.ty(), OperationType::Subscription),
            "execute shouldn't be called for subscriptions"
        );

        if let Some(response) = self.response_if_root_errors() {
            return response;
        }

        OperationExecution {
            coordinator: &self,
            futures: ExecutorFutureSet::new(),
            state: self.new_execution_state(),
            response: ResponseBuilder::new(self.operation.root_object_id),
        }
        .execute()
        .await
    }

    pub async fn execute_subscription(self, mut responses: impl ResponseSender) {
        assert!(matches!(self.operation.ty(), OperationType::Subscription));

        if let Some(response) = self.response_if_root_errors() {
            let _ = responses.send(response).await;
            return;
        }

        let (state, subscription_plan_id) = {
            let mut state = self.new_execution_state();
            let id = state.pop_subscription_plan_id();
            (state, id)
        };
        let response_object_set_ids = self
            .plan_walker(subscription_plan_id)
            .logical_plan()
            .response_blueprint()
            .output_ids;
        let new_execution = || {
            let mut response = ResponseBuilder::new(self.operation.root_object_id);
            OperationRootPlanExecution {
                root_response_part: response.new_part(
                    Arc::new(response.root_response_object().into_iter().collect()),
                    response_object_set_ids,
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
        let plan = self.plan_walker(plan_id);
        let input = SubscriptionInput { ctx: self.ctx, plan };
        let executor = self.operation[plan_id]
            .prepared_executor
            .new_subscription_executor(input)?;
        executor.execute(new_execution).await
    }

    fn response_if_root_errors(&self) -> Option<Response> {
        if self.operation.query_modifications.root_error_ids.is_empty() {
            return None;
        }
        let mut response = ResponseBuilder::new(self.operation.root_object_id);
        response.push_root_errors(
            self.operation
                .query_modifications
                .root_error_ids
                .iter()
                .map(|&id| self.operation[id].clone()),
        );
        let schema = self.engine.schema.clone();
        let operation = self.operation.prepared.clone();
        Some(response.build(schema, operation))
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
    async fn execute(self, mut responses: impl ResponseSender) {
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

struct OperationExecution<'ctx, R: Runtime> {
    coordinator: &'ctx ExecutionCoordinator<'ctx, R>,
    futures: ExecutorFutureSet<'ctx>,
    state: OperationExecutionState<'ctx>,
    response: ResponseBuilder,
}

impl<'ctx, R: Runtime> std::ops::Deref for OperationExecution<'ctx, R> {
    type Target = ExecutionContext<'ctx, R>;
    fn deref(&self) -> &Self::Target {
        &self.coordinator.ctx
    }
}

impl<'ctx, R: Runtime> OperationExecution<'ctx, R> {
    /// Runs a single execution to completion, returning its response
    async fn execute(mut self) -> Response {
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
            let (first_edge, default_object) = self.get_first_edge_and_default_object(plan_id);
            match result {
                Ok(part) => {
                    tracing::trace!(%plan_id, "Succeeded");

                    for (entity_location, response_object_refs) in
                        self.response.ingest(part, first_edge, default_object)
                    {
                        self.state.push_response_objects(entity_location, response_object_refs);
                    }

                    for plan_id in self.state.get_next_executable_plans(plan_id) {
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

        let schema = self.engine.schema.clone();
        let operation = self.operation.prepared.clone();
        self.response.build(schema, operation)
    }

    fn get_first_edge_and_default_object(
        &self,
        plan_id: ExecutionPlanId,
    ) -> (ResponseEdge, Option<Vec<(ResponseEdge, ResponseValue)>>) {
        let shape_id = self
            .coordinator
            .plan_walker(plan_id)
            .logical_plan()
            .response_blueprint()
            .concrete_shape_id;
        let shapes = &self.operation.response_blueprint.shapes;
        let shape = &shapes[shape_id];
        let first_edge = shapes[shape.field_shape_ids]
            .iter()
            .map(|field| field.edge)
            .min()
            .or_else(|| shape.typename_response_edges.iter().min().copied())
            .expect("Selection set without any fields?");

        let mut fields = Vec::new();
        if !shape.typename_response_edges.is_empty() {
            if let ObjectIdentifier::Known(object_id) = shape.identifier {
                let name: ResponseValue = self.schema().walk(object_id).as_ref().name.into();
                fields.extend(shape.typename_response_edges.iter().map(|&edge| (edge, name.clone())))
            } else {
                return (first_edge, None);
            }
        }
        for field in &shapes[shape.field_shape_ids] {
            if field.wrapping.is_required() {
                return (first_edge, None);
            }
            fields.push((field.edge, ResponseValue::Null))
        }

        (first_edge, Some(fields))
    }

    fn spawn_executor(&mut self, plan_id: ExecutionPlanId) {
        tracing::trace!(%plan_id, "Starting plan");
        let root_response_object_refs = self.state.get_root_response_object_refs(&self.response, plan_id);

        tracing::trace!(%plan_id, "Found {} root response objects", root_response_object_refs.len());
        if root_response_object_refs.is_empty() {
            return;
        }

        let plan = self.coordinator.plan_walker(plan_id);
        let response_part = self.response.new_part(
            root_response_object_refs.clone(),
            plan.logical_plan().response_blueprint().output_ids,
        );
        let input = ExecutorInput {
            ctx: self.coordinator.ctx,
            plan,
            root_response_objects: self.response.read(
                plan.schema(),
                root_response_object_refs.clone(),
                &self.operation[plan_id].requires,
            ),
        };

        match self.operation[plan_id].prepared_executor.new_executor(input) {
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
