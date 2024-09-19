use std::sync::Arc;

use async_runtime::make_send_on_wasm;
use engine_parser::types::OperationType;
use futures::{stream::FuturesOrdered, Future, FutureExt, Stream};
use futures_util::{
    future::BoxFuture,
    stream::{BoxStream, FuturesUnordered},
    StreamExt,
};
use grafbase_telemetry::graphql::GraphqlResponseStatus;
use runtime::hooks::ExecutedOperationBuilder;
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

pub(crate) trait ResponseSender: Send {
    type Error;
    fn send(&mut self, response: Response) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    pub async fn execute_query_or_mutation(mut self, operation: ExecutableOperation) -> Response {
        let background_futures: FuturesUnordered<_> =
            std::mem::take(&mut self.background_futures).into_iter().collect();
        let background_fut = background_futures.collect::<Vec<_>>();

        tracing::trace!("Starting execution...");
        if operation.query_modifications.root_error_ids.is_empty() {
            let ctx = ExecutionContext {
                engine: self.engine,
                operation: &operation,
                request_context: self.request_context,
                hooks_context: &self.hooks_context,
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

    pub async fn execute_subscription(mut self, operation: ExecutableOperation, mut responses: impl ResponseSender) {
        let background_futures: FuturesUnordered<_> =
            std::mem::take(&mut self.background_futures).into_iter().collect();
        let background_fut = background_futures.collect::<Vec<_>>();

        tracing::trace!("Starting execution...");
        if operation.query_modifications.root_error_ids.is_empty() {
            let ctx = ExecutionContext {
                engine: self.engine,
                operation: &operation,
                request_context: self.request_context,
                hooks_context: &self.hooks_context,
            };

            let subscription_fut = ctx.execute_subscription(self.executed_operation_builder, responses);

            futures_util::join!(subscription_fut, background_fut);
        } else {
            let response_fut = self.response_for_root_errors(operation);
            let (response, _) = futures_util::join!(response_fut, background_fut);
            responses.send(response).await.ok();
        }
    }

    async fn response_for_root_errors(self, operation: ExecutableOperation) -> Response {
        let executed_operation = self.executed_operation_builder.clone().build(
            operation.attributes.name.original(),
            &operation.attributes.sanitized_query,
            GraphqlResponseStatus::FieldError {
                count: operation.query_modifications.root_error_ids.len() as u64,
                data_is_null: true,
            },
        );

        match self.hooks().on_operation_response(executed_operation).await {
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

    async fn execute(self, executed_operation_builder: ExecutedOperationBuilder) -> Response {
        assert!(
            !matches!(self.operation.ty(), OperationType::Subscription),
            "execute shouldn't be called for subscriptions"
        );

        OperationExecution {
            futures: ExecutionPlanFutureSet::new(),
            state: self.new_execution_state(),
            executed_operation_builder,
            response: ResponseBuilder::new(self.operation.root_object_id),
            ctx: self,
        }
        .run()
        .await
    }

    async fn execute_subscription(
        self,
        executed_operation_builder: ExecutedOperationBuilder,
        responses: impl ResponseSender,
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
            initial_executed_operation_builder: executed_operation_builder,
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
    initial_executed_operation_builder: ExecutedOperationBuilder,
    stream: S,
}

impl<'ctx, 'exec, R: Runtime, S> SubscriptionExecution<'ctx, R, S>
where
    'ctx: 'exec,
    S: Stream<Item = ExecutionResult<SubscriptionResponse>> + Send + 'exec,
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
                        Ok(SubscriptionResponse {
                            response,
                            root_subgraph_response,
                        }) => {
                            let mut operation_execution = OperationExecution {
                                futures: ExecutionPlanFutureSet::new(),
                                ctx: self.ctx,
                                executed_operation_builder: self.initial_executed_operation_builder.clone(),
                                state: self.initial_state.clone(),
                                response,
                            };

                            operation_execution.futures.push_result(ExecutionPlanResult {
                                plan_id: self.subscription_plan_id,
                                result: Ok(root_subgraph_response),
                                on_subgraph_response_hook_output: None,
                            });

                            response_futures.push_back(operation_execution.run());
                        }
                        Err(err) => {
                            let operation = self.ctx.operation.prepared.clone();
                            let executed_operation = self.initial_executed_operation_builder.clone().build(
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

struct OperationExecution<'ctx, 'exec, R: Runtime> {
    ctx: ExecutionContext<'ctx, R>,
    futures: ExecutionPlanFutureSet<'exec>,
    executed_operation_builder: ExecutedOperationBuilder,
    state: OperationExecutionState<'ctx>,
    response: ResponseBuilder,
}

impl<'ctx, 'exec, R: Runtime> std::ops::Deref for OperationExecution<'ctx, 'exec, R> {
    type Target = ExecutionContext<'ctx, R>;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<'ctx, 'exec, R: Runtime> OperationExecution<'ctx, 'exec, R>
where
    'ctx: 'exec,
{
    /// Runs a single execution to completion, returning its response
    async fn run(mut self) -> Response {
        for plan_id in self.state.get_executable_plans() {
            self.spawn_resolver(plan_id);
        }

        while let Some(ExecutionPlanResult {
            plan_id,
            result,
            on_subgraph_response_hook_output,
        }) = self.futures.next().await
        {
            // Retrieving the first edge (response key) appearing in the query to provide a better
            // error path if necessary.
            let (any_edge, default_fields) = self.get_first_edge_and_default_object(plan_id);
            match result {
                Ok(subgraph_response) => {
                    tracing::trace!(%plan_id, "Succeeded");

                    let tracked_response_object_sets =
                        self.response.ingest(subgraph_response, any_edge, default_fields);

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
                        self.spawn_resolver(plan_id);
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
        }

        let schema = self.ctx.engine.schema.clone();
        let operation = self.ctx.operation.prepared.clone();
        let executed_operation = self.executed_operation_builder.build(
            operation.attributes.name.original(),
            &operation.attributes.sanitized_query,
            self.response.graphql_status(),
        );

        match self.ctx.hooks().on_operation_response(executed_operation).await {
            Ok(output) => self.response.build(schema, operation, output),
            Err(err) => Response::execution_error(operation, None, [err]),
        }
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

    fn spawn_resolver(&mut self, plan_id: ExecutionPlanId) {
        tracing::trace!(%plan_id, "Starting plan");
        let root_response_object_set = Arc::new(self.state.get_input(&self.response, plan_id));

        tracing::trace!(%plan_id, "Found {} root response objects", root_response_object_set.len());
        if root_response_object_set.is_empty() {
            return;
        }

        self.futures.push_fut({
            let span = tracing::debug_span!("resoler", "plan_id" = usize::from(plan_id)).entered();

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
            make_send_on_wasm(fut.instrument(span)).boxed()
        });
    }
}

struct ExecutionPlanFutureSet<'exec> {
    futures: FuturesUnordered<BoxFuture<'exec, ExecutionPlanResult>>,
}

impl<'exec> ExecutionPlanFutureSet<'exec> {
    fn new() -> Self {
        Self {
            futures: FuturesUnordered::new(),
        }
    }

    fn push_fut(&mut self, fut: BoxFuture<'exec, ExecutionPlanResult>) {
        self.futures.push(fut);
    }

    fn push_result(&mut self, result: ExecutionPlanResult) {
        self.futures.push(Box::pin(async move { result }));
    }

    async fn next(&mut self) -> Option<ExecutionPlanResult> {
        self.futures.next().await
    }
}

pub(crate) struct ExecutionPlanResult {
    plan_id: ExecutionPlanId,
    result: Result<SubgraphResponse, (Arc<InputResponseObjectSet>, ExecutionError)>,
    on_subgraph_response_hook_output: Option<Vec<u8>>,
}
