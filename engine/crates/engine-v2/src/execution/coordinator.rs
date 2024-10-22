use std::{collections::VecDeque, pin::Pin, sync::Arc};

use async_runtime::make_send_on_wasm;
use engine_parser::types::OperationType;
use futures::{stream::FuturesOrdered, Future, FutureExt, SinkExt, Stream};
use futures_util::{
    future::BoxFuture,
    stream::{BoxStream, FuturesUnordered},
    StreamExt,
};
use grafbase_telemetry::graphql::GraphqlResponseStatus;
use runtime::hooks::{ExecutedOperationBuilder, Hooks};
use tracing::Instrument;

use crate::{
    execution::{ExecutableOperation, ExecutionContext},
    operation::PlanWalker,
    response::{
        InputResponseObjectSet, ObjectIdentifier, Response, ResponseBuilder, ResponseEdge, ResponseObjectField,
        ResponseValue, SubgraphResponse, SubgraphResponseRefMut,
    },
    sources::ResolverResult,
    Runtime,
};

use super::{state::OperationExecutionState, ExecutionError, ExecutionPlanId, ExecutionResult, PreExecutionContext};

pub(crate) trait ResponseSender<O>: Send {
    type Error;
    fn send(&mut self, response: Response<O>) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    pub async fn execute_query_or_mutation(
        mut self,
        operation: ExecutableOperation,
    ) -> Response<<R::Hooks as Hooks>::OnOperationResponseOutput> {
        let background_futures: FuturesUnordered<_> =
            std::mem::take(&mut self.background_futures).into_iter().collect();
        let background_fut = background_futures.collect::<Vec<_>>();

        tracing::trace!("Starting execution...");
        if operation.query_modifications.root_error_ids.is_empty() {
            let operation = Arc::new(operation);
            let hooks_context = Arc::new(self.hooks_context);
            let ctx = ExecutionContext {
                engine: self.engine,
                operation: &operation,
                request_context: self.request_context,
                hooks_context: &hooks_context,
            };
            let response_fut = ctx.execute(self.executed_operation_builder);
            let (response, _) = futures_util::join!(response_fut, background_fut);
            response
        } else {
            let response_fut = self.response_for_root_errors(operation);
            let (response, _) = futures_util::join!(response_fut, background_fut);
            response
        }
    }

    pub async fn execute_subscription(
        mut self,
        operation: ExecutableOperation,
        mut responses: impl ResponseSender<<R::Hooks as Hooks>::OnOperationResponseOutput>,
    ) {
        let background_futures: FuturesUnordered<_> =
            std::mem::take(&mut self.background_futures).into_iter().collect();
        let background_fut = background_futures.collect::<Vec<_>>();

        tracing::trace!("Starting execution...");
        if operation.query_modifications.root_error_ids.is_empty() {
            let operation = Arc::new(operation);
            let hooks_context = Arc::new(self.hooks_context);
            let ctx = ExecutionContext {
                engine: self.engine,
                operation: &operation,
                request_context: self.request_context,
                hooks_context: &hooks_context,
            };

            let subscription_fut = ctx.execute_subscription(self.executed_operation_builder, responses);

            futures_util::join!(subscription_fut, background_fut);
        } else {
            let response_fut = self.response_for_root_errors(operation);
            let (response, _) = futures_util::join!(response_fut, background_fut);
            responses.send(response).await.ok();
        }
    }

    async fn response_for_root_errors(
        self,
        operation: ExecutableOperation,
    ) -> Response<<R::Hooks as Hooks>::OnOperationResponseOutput> {
        let executed_operation = self.executed_operation_builder.build(
            operation.attributes.name.original(),
            &operation.attributes.sanitized_query,
            GraphqlResponseStatus::FieldError {
                count: operation.query_modifications.root_error_ids.len() as u64,
                data_is_null: true,
            },
        );

        match self
            .engine
            .runtime
            .hooks()
            .on_operation_response(&self.hooks_context, executed_operation)
            .await
        {
            Ok(output) => Response::execution_error(
                operation.prepared.clone(),
                Some(output),
                operation
                    .query_modifications
                    .root_error_ids
                    .iter()
                    .copied()
                    .map(|id| operation.query_modifications[id].clone()),
            ),
            Err(err) => Response::execution_error(operation.prepared.clone(), None, [err]),
        }
    }
}

impl<'ctx, R: Runtime> ExecutionContext<'ctx, R> {
    fn new_execution_state(&self) -> OperationExecutionState<'ctx> {
        OperationExecutionState::new(&self.engine.schema, self.operation)
    }

    pub(super) fn plan_walker(&self, plan_id: ExecutionPlanId) -> PlanWalker<'ctx, ()> {
        PlanWalker {
            schema: &self.engine.schema,
            operation: self.operation,
            variables: &self.operation.variables,
            query_modifications: &self.operation.query_modifications,
            logical_plan_id: self.operation[plan_id].logical_plan_id,
            item: (),
        }
    }

    async fn execute(
        self,
        executed_operation_builder: ExecutedOperationBuilder<<R::Hooks as Hooks>::OnSubgraphResponseOutput>,
    ) -> Response<<R::Hooks as Hooks>::OnOperationResponseOutput> {
        assert!(
            !matches!(self.operation.ty(), OperationType::Subscription),
            "execute shouldn't be called for subscriptions"
        );

        OperationExecution {
            state: self.new_execution_state(),
            executed_operation_builder,
            response: ResponseBuilder::new(self.operation.root_object_id),
            ctx: self,
        }
        .run(VecDeque::new())
        .await
    }

    async fn execute_subscription(
        self,
        executed_operation_builder: ExecutedOperationBuilder<<R::Hooks as Hooks>::OnSubgraphResponseOutput>,
        responses: impl ResponseSender<<R::Hooks as Hooks>::OnOperationResponseOutput>,
    ) {
        assert!(matches!(self.operation.ty(), OperationType::Subscription));

        let (initial_state, subscription_plan_id) = {
            let mut state = self.new_execution_state();
            let id = state.pop_subscription_plan_id();
            (state, id)
        };

        let stream = match self.build_subscription_stream(subscription_plan_id).await {
            Ok(stream) => stream,
            Err(error) => Box::pin(futures_util::stream::iter(std::iter::once(Err(error)))),
        };

        SubscriptionExecution {
            ctx: self,
            subscription_plan_id,
            first_executed_operation_builder: Some(executed_operation_builder),
            initial_state,
            stream,
        }
        .execute(responses)
        .await
    }

    async fn build_subscription_stream<'exec>(
        self,
        subscription_plan_id: ExecutionPlanId,
    ) -> ExecutionResult<BoxStream<'exec, ExecutionResult<SubscriptionResponse>>>
    where
        'ctx: 'exec,
    {
        let plan = self.plan_walker(subscription_plan_id);
        self.operation[subscription_plan_id]
            .resolver
            .execute_subscription(self, plan, move || self.new_subscription_response(subscription_plan_id))
            .await
    }

    fn new_subscription_response(&self, subscription_plan_id: ExecutionPlanId) -> SubscriptionResponse {
        let mut response = ResponseBuilder::new(self.operation.root_object_id);

        let tracked_response_object_set_ids = self
            .plan_walker(subscription_plan_id)
            .logical_plan()
            .response_blueprint()
            .output_ids;

        let root_response_object_set = Arc::new(
            InputResponseObjectSet::default()
                .with_response_objects(Arc::new(response.root_response_object().into_iter().collect())),
        );

        let root_subgraph_response = response.new_subgraph_response(
            self.operation[subscription_plan_id].logical_plan_id,
            root_response_object_set,
            tracked_response_object_set_ids,
        );

        SubscriptionResponse {
            response,
            root_subgraph_response,
        }
    }
}

struct SubscriptionExecution<'ctx, R: Runtime, S> {
    ctx: ExecutionContext<'ctx, R>,
    subscription_plan_id: ExecutionPlanId,
    initial_state: OperationExecutionState<'ctx>,
    first_executed_operation_builder: Option<ExecutedOperationBuilder<<R::Hooks as Hooks>::OnSubgraphResponseOutput>>,
    stream: S,
}

impl<'ctx, 'exec, R: Runtime, S> SubscriptionExecution<'ctx, R, S>
where
    'ctx: 'exec,
    S: Stream<Item = ExecutionResult<SubscriptionResponse>> + Send + 'exec,
{
    async fn execute(mut self, mut responses: impl ResponseSender<<R::Hooks as Hooks>::OnOperationResponseOutput>) {
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
                    let executed_operation_builder =
                        self.first_executed_operation_builder
                            .take()
                            .unwrap_or_else(|| ExecutedOperationBuilder {
                                start_time: web_time::Instant::now(),
                                prepare_duration: Some(Default::default()),
                                cached_plan: true,
                                on_subgraph_response_outputs: Vec::new(),
                            });
                    match execution {
                        Ok(SubscriptionResponse {
                            response,
                            root_subgraph_response,
                        }) => {
                            let mut results = VecDeque::new();
                            results.push_back(ExecutionPlanResult {
                                plan_id: self.subscription_plan_id,
                                result: Ok(root_subgraph_response),
                                on_subgraph_response_hook_output: None,
                            });

                            let operation_execution = OperationExecution {
                                ctx: self.ctx,
                                executed_operation_builder,
                                state: self.initial_state.clone(),
                                response,
                            };

                            response_futures.push_back(operation_execution.run(results));
                        }
                        Err(err) => {
                            let operation = self.ctx.operation.prepared.clone();
                            let executed_operation = executed_operation_builder.build(
                                operation.attributes.name.original(),
                                &operation.attributes.sanitized_query,
                                GraphqlResponseStatus::FieldError {
                                    count: 1,
                                    data_is_null: true,
                                },
                            );

                            let response = match self.ctx.hooks().on_operation_response(executed_operation).await {
                                Ok(output) => Response::execution_error(operation, Some(output), [err]),
                                Err(err) => Response::execution_error(operation, None, [err]),
                            };

                            if responses.send(response).await.is_err() {
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

pub(crate) struct SubscriptionResponse {
    response: ResponseBuilder,
    root_subgraph_response: SubgraphResponse,
}

impl SubscriptionResponse {
    pub fn as_mut(&mut self) -> SubgraphResponseRefMut<'_> {
        self.root_subgraph_response.as_mut()
    }
}

struct OperationExecution<'ctx, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    executed_operation_builder: ExecutedOperationBuilder<<R::Hooks as Hooks>::OnSubgraphResponseOutput>,
    state: OperationExecutionState<'ctx>,
    response: ResponseBuilder,
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

impl<'ctx, R: Runtime> OperationExecution<'ctx, R> {
    /// Runs a single execution to completion, returning its response
    async fn run(
        mut self,
        mut results: VecDeque<ExecutionPlanResult<<R::Hooks as Hooks>::OnSubgraphResponseOutput>>,
    ) -> Response<<R::Hooks as Hooks>::OnOperationResponseOutput> {
        let futures = FuturesUnordered::new();
        for plan_id in self.state.get_executable_plans() {
            if let Some(fut) = self.create_plan_execution_future(plan_id) {
                futures.push(fut);
            }
        }

        let mut state = State::Execution(self);

        futures_util::pin_mut!(futures);
        let self_ = loop {
            state = match state {
                State::Ingestion(mut ingestion_fut) => {
                    let mut fut = unsafe { Pin::new_unchecked(&mut ingestion_fut) };
                    let next_task = futures_util::select_biased! {
                        ingestion_result = fut => Ok(ingestion_result),
                        execution_result = futures.next() => Err(execution_result),
                    };
                    match next_task {
                        Ok((self_, next_futures)) => {
                            for fut in next_futures {
                                futures.push(fut);
                            }
                            State::Execution(self_)
                        }
                        Err(Some(result)) => {
                            results.push_back(result);
                            State::Ingestion(ingestion_fut)
                        }
                        Err(None) => {
                            let (self_, next_futures) = ingestion_fut.await;
                            for fut in next_futures {
                                futures.push(fut)
                            }
                            State::Execution(self_)
                        }
                    }
                }
                State::Execution(self_) => {
                    if let Some(result) = results.pop_front() {
                        State::Ingestion(self_.ingest_execution_result(result).fuse())
                    } else if let Some(result) = futures.next().await {
                        results.push_back(result);
                        State::Execution(self_)
                    } else {
                        break self_;
                    }
                }
            };
        };

        let schema = self_.ctx.engine.schema.clone();
        let operation = self_.ctx.operation.prepared.clone();
        let executed_operation = self_.executed_operation_builder.build(
            operation.attributes.name.original(),
            &operation.attributes.sanitized_query,
            self_.response.graphql_status(),
        );

        match self_.ctx.hooks().on_operation_response(executed_operation).await {
            Ok(output) => self_.response.build(schema, operation, output),
            Err(err) => Response::execution_error(operation, None, [err]),
        }
    }

    async fn ingest_execution_result<'exec>(
        mut self,
        ExecutionPlanResult {
            plan_id,
            result,
            on_subgraph_response_hook_output,
        }: ExecutionPlanResult<<R::Hooks as Hooks>::OnSubgraphResponseOutput>,
    ) -> (
        Self,
        Vec<BoxFuture<'exec, ExecutionPlanResult<<R::Hooks as Hooks>::OnSubgraphResponseOutput>>>,
    )
    where
        'ctx: 'exec,
    {
        let mut next_futures = Vec::new();

        // Retrieving the first edge (response key) appearing in the query to provide a better
        // error path if necessary.
        let (any_edge, default_fields) = self.get_first_edge_and_default_object(plan_id);
        match result {
            Ok(subgraph_response) => {
                tracing::trace!(%plan_id, "Succeeded");

                let tracked_response_object_sets = self.response.ingest(subgraph_response, any_edge, default_fields);

                for (set_id, response_object_refs) in tracked_response_object_sets.into_iter() {
                    self.state.push_response_objects(set_id, response_object_refs);
                }

                let response_modifier_executor_ids = self.state.get_next_executable_response_modifiers(plan_id);

                for id in &response_modifier_executor_ids {
                    self.ctx
                        .execute_response_modifier(&mut self.state, &mut self.response, *id)
                        .await;
                }

                for plan_id in self
                    .state
                    .get_next_executable_plans(plan_id, response_modifier_executor_ids)
                {
                    if let Some(fut) = self.create_plan_execution_future(plan_id) {
                        next_futures.push(fut);
                    }
                }
            }
            Err((root_response_object_set, error)) => {
                tracing::trace!(%plan_id, "Failed");
                self.response
                    .propagate_execution_error(root_response_object_set, error, any_edge, default_fields);
            }
        }

        if let Some(output) = on_subgraph_response_hook_output {
            self.executed_operation_builder.push_on_subgraph_response_output(output);
        }

        (self, next_futures)
    }

    fn get_first_edge_and_default_object(
        &self,
        plan_id: ExecutionPlanId,
    ) -> (ResponseEdge, Option<Vec<ResponseObjectField>>) {
        let shape_id = self
            .ctx
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
                let name: ResponseValue = self.schema().walk(object_id).as_ref().name_id.into();
                fields.extend(shape.typename_response_edges.iter().map(|&edge| ResponseObjectField {
                    edge,
                    required_field_id: None,
                    value: name.clone(),
                }))
            } else {
                return (first_edge, None);
            }
        }
        for field_shape in &shapes[shape.field_shape_ids] {
            if field_shape.wrapping.is_required() {
                return (first_edge, None);
            }
            fields.push(ResponseObjectField {
                edge: field_shape.edge,
                required_field_id: field_shape.required_field_id,
                value: ResponseValue::Null,
            })
        }

        (first_edge, Some(fields))
    }

    fn create_plan_execution_future<'exec>(
        &mut self,
        plan_id: ExecutionPlanId,
    ) -> Option<BoxFuture<'exec, ExecutionPlanResult<<R::Hooks as Hooks>::OnSubgraphResponseOutput>>>
    where
        'ctx: 'exec,
    {
        tracing::trace!(%plan_id, "Starting plan");
        let root_response_object_set = Arc::new(self.state.get_input(&self.response, plan_id));

        tracing::trace!(%plan_id, "Found {} root response objects", root_response_object_set.len());
        if root_response_object_set.is_empty() {
            return None;
        }

        let span = tracing::debug_span!("resolver", "plan_id" = usize::from(plan_id)).entered();

        let plan = self.ctx.plan_walker(plan_id);

        let subgraph_response = self.response.new_subgraph_response(
            plan.logical_plan_id,
            Arc::clone(&root_response_object_set),
            plan.logical_plan().response_blueprint().output_ids,
        );

        let root_response_objects = self.response.read(
            self.ctx.schema(),
            &self.ctx.operation.response_views,
            Arc::clone(&root_response_object_set),
            self.operation[plan_id].requires,
        );

        let fut = self.operation[plan_id]
            .resolver
            .execute(self.ctx, plan, root_response_objects, subgraph_response)
            .map(
                move |ResolverResult {
                          execution,
                          on_subgraph_response_hook_output,
                      }| ExecutionPlanResult {
                    plan_id,
                    result: execution.map_err(|err| (root_response_object_set, err)),
                    on_subgraph_response_hook_output,
                },
            );

        let span = span.exit();
        Some(make_send_on_wasm(fut.instrument(span)).boxed())
    }
}

pub(crate) struct ExecutionPlanResult<OnSubgraphResponseHookOutput> {
    plan_id: ExecutionPlanId,
    result: Result<SubgraphResponse, (Arc<InputResponseObjectSet>, ExecutionError)>,
    on_subgraph_response_hook_output: Option<OnSubgraphResponseHookOutput>,
}
