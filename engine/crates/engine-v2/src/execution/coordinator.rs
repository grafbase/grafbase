use std::{borrow::Cow, sync::Arc};

use async_runtime::make_send_on_wasm;
use engine::RequestHeaders;
use engine_parser::types::OperationType;
use futures_util::{
    future::BoxFuture,
    stream::{BoxStream, FuturesUnordered},
    Future, SinkExt, StreamExt,
};
use grafbase_tracing::span::{GqlRecorderSpanExt, GqlRequestAttributes, GqlResponseAttributes};
use tracing::Span;

use crate::{
    execution::ExecutionContext,
    plan::{OperationExecutionState, OperationPlan, PlanId},
    request::{OpInputValues, Operation},
    response::{ExecutionMetadata, GraphqlError, Response, ResponseBoundaryItem, ResponseBuilder, ResponsePart},
    sources::{Executor, ExecutorInput, SubscriptionExecutor, SubscriptionInput},
    Engine,
};

use super::ExecutionResult;

pub type ResponseSender = futures::channel::mpsc::Sender<Response>;

pub(crate) struct ExecutionCoordinator {
    engine: Arc<Engine>,
    operation_plan: Arc<OperationPlan>,
    input_values: OpInputValues,
    request_headers: RequestHeaders,
}

impl ExecutionCoordinator {
    pub fn new(
        engine: Arc<Engine>,
        operation_plan: Arc<OperationPlan>,
        input_values: OpInputValues,
        request_headers: RequestHeaders,
    ) -> Self {
        Self {
            engine,
            operation_plan,
            input_values,
            request_headers,
        }
    }

    pub fn operation(&self) -> &Operation {
        &self.operation_plan
    }

    pub async fn execute(self) -> Response {
        let gql_span = Span::current();
        gql_span.record_gql_request(GqlRequestAttributes {
            operation_type: self.operation().ty.as_ref(),
            operation_name: self.operation().name.as_deref(),
        });

        assert!(
            !matches!(self.operation_plan.ty, OperationType::Subscription),
            "execute shouldn't be called for subscriptions"
        );

        let response = OperationExecution {
            coordinator: &self,
            futures: ExecutorFutureSet::new(),
            state: self.operation_plan.new_execution_state(),
            response: ResponseBuilder::new(self.operation_plan.root_object_id),
        }
        .execute()
        .await;

        gql_span.record_gql_response(GqlResponseAttributes {
            has_errors: response.has_errors(),
        });

        response
    }

    pub async fn execute_subscription(self, mut responses: ResponseSender) {
        assert!(matches!(self.operation_plan.ty, OperationType::Subscription));

        let current_span = Span::current();
        current_span.record_gql_request(GqlRequestAttributes {
            operation_type: self.operation().ty.as_ref(),
            operation_name: self.operation().name.as_deref(),
        });
        let (state, subscription_plan_id) = {
            let mut state = self.operation_plan.new_execution_state();
            let id = state.pop_subscription_plan_id();
            (state, id)
        };
        let root_plan_boundary_ids = self
            .operation_plan
            .plan_walker(&self.engine.schema, subscription_plan_id, None)
            .output()
            .boundary_ids;
        let new_execution = || {
            let mut response = ResponseBuilder::new(self.operation_plan.root_object_id);
            OperationRootPlanExecution {
                root_response_part: response.new_part(root_plan_boundary_ids),
                operation_execution: OperationExecution {
                    coordinator: &self,
                    futures: ExecutorFutureSet::new(),
                    state: state.clone(),
                    response,
                },
            }
        };

        let mut stream = match self
            .build_subscription_stream(subscription_plan_id, new_execution)
            .await
        {
            Ok(stream) => stream,
            Err(error) => {
                responses
                    .send(
                        ResponseBuilder::new(self.operation_plan.root_object_id)
                            .with_error(error)
                            .build(
                                self.engine.schema.clone(),
                                self.operation_plan.clone(),
                                ExecutionMetadata::build(&self.operation_plan),
                            ),
                    )
                    .await
                    .ok();
                return;
            }
        };

        while let Some(OperationRootPlanExecution {
            mut operation_execution,
            root_response_part,
        }) = stream.next().await
        {
            operation_execution.futures.push(async move {
                ExecutorFutureResult {
                    result: Ok(root_response_part),
                    plan_id: subscription_plan_id,
                }
            });
            let response = operation_execution.execute().await;
            if responses.send(response).await.is_err() {
                return;
            }
        }
    }

    async fn build_subscription_stream<'s, 'ctx>(
        &'s self,
        plan_id: PlanId,
        new_execution: impl Fn() -> OperationRootPlanExecution<'ctx> + Send + 'ctx,
    ) -> Result<BoxStream<'ctx, OperationRootPlanExecution<'ctx>>, GraphqlError>
    where
        's: 'ctx,
    {
        let executor = self.build_subscription_executor(plan_id)?;
        Ok(executor.execute(new_execution).await?)
    }

    fn build_subscription_executor(&self, plan_id: PlanId) -> ExecutionResult<SubscriptionExecutor<'_>> {
        let execution_plan = &self.operation_plan[plan_id];
        let plan = self
            .operation_plan
            .plan_walker(&self.engine.schema, plan_id, Some(&self.input_values));
        let input = SubscriptionInput {
            ctx: ExecutionContext {
                engine: self.engine.as_ref(),
                request_headers: &self.request_headers,
            },
            plan,
        };
        execution_plan.new_subscription_executor(input)
    }
}

pub struct OperationRootPlanExecution<'ctx> {
    operation_execution: OperationExecution<'ctx>,
    root_response_part: ResponsePart,
}

impl OperationRootPlanExecution<'_> {
    pub fn root_response_part(&mut self) -> &mut ResponsePart {
        &mut self.root_response_part
    }

    pub fn root_response_boundary_item(&self) -> ResponseBoundaryItem {
        self.operation_execution
            .response
            .root_response_boundary_item()
            .expect("a fresh response should always have a root")
    }
}

pub struct OperationExecution<'ctx> {
    coordinator: &'ctx ExecutionCoordinator,
    futures: ExecutorFutureSet<'ctx>,
    state: OperationExecutionState,
    response: ResponseBuilder,
}

impl<'ctx> OperationExecution<'ctx> {
    /// Runs a single execution to completion, returning its response
    async fn execute(mut self) -> Response {
        for plan_id in self.state.get_executable_plans() {
            tracing::debug!(%plan_id, "Starting plan");
            match self.build_executor(plan_id) {
                Ok(Some(executor)) => self.futures.execute(plan_id, executor),
                Ok(None) => (),
                Err(error) => self.response.push_error(error),
            }
        }

        while let Some(ExecutorFutureResult { result, plan_id }) = self.futures.next().await {
            let output = match result {
                Ok(output) => output,
                Err(err) => {
                    tracing::debug!(%plan_id, "Failed");
                    self.response.push_error(err);
                    continue;
                }
            };
            tracing::debug!(%plan_id, "Succeeded");

            // Ingesting data first to propagate errors and next plans likely rely on it
            for (plan_bounday_id, boundary) in self.response.ingest(output) {
                self.state.push_boundary_items(plan_bounday_id, boundary);
            }

            for plan_id in self
                .state
                .get_next_executable_plans(&self.coordinator.operation_plan, plan_id)
            {
                match self.build_executor(plan_id) {
                    Ok(Some(executor)) => self.futures.execute(plan_id, executor),
                    Ok(None) => (),
                    Err(error) => self.response.push_error(error),
                }
            }
        }

        self.response.build(
            self.coordinator.engine.schema.clone(),
            self.coordinator.operation_plan.clone(),
            ExecutionMetadata::build(&self.coordinator.operation_plan),
        )
    }

    fn build_executor(&mut self, plan_id: PlanId) -> ExecutionResult<Option<Executor<'ctx>>> {
        let operation: &'ctx OperationPlan = &self.coordinator.operation_plan;
        let engine = self.coordinator.engine.as_ref();
        let response_boundary_items =
            self.state
                .retrieve_boundary_items(&engine.schema, operation, &self.response, plan_id);

        tracing::debug!(%plan_id, "Found {} response boundary items", response_boundary_items.len());
        if response_boundary_items.is_empty() {
            return Ok(None);
        }

        let execution_plan = &operation[plan_id];
        let plan =
            self.coordinator
                .operation_plan
                .plan_walker(&engine.schema, plan_id, Some(&self.coordinator.input_values));
        let response_part = self.response.new_part(plan.output().boundary_ids);
        let input = ExecutorInput {
            ctx: ExecutionContext {
                engine,
                request_headers: &self.coordinator.request_headers,
            },
            plan,
            boundary_objects_view: self.response.read(
                plan.schema(),
                response_boundary_items,
                plan.input()
                    .map(|input| Cow::Borrowed(&input.selection_set))
                    .unwrap_or_default(),
            ),
            response_part,
        };

        tracing::debug!("{:#?}", input.plan.collected_selection_set());
        execution_plan.new_executor(input).map(Some)
    }
}

pub struct ExecutorFutureSet<'a>(FuturesUnordered<BoxFuture<'a, ExecutorFutureResult>>);

impl<'a> ExecutorFutureSet<'a> {
    fn new() -> Self {
        ExecutorFutureSet(FuturesUnordered::new())
    }

    fn execute(&mut self, plan_id: PlanId, executor: Executor<'a>) {
        self.push(make_send_on_wasm(async move {
            let result = executor.execute().await;
            ExecutorFutureResult { plan_id, result }
        }))
    }

    fn push(&mut self, fut: impl Future<Output = ExecutorFutureResult> + Send + 'a) {
        self.0.push(Box::pin(fut));
    }

    async fn next(&mut self) -> Option<ExecutorFutureResult> {
        self.0.next().await
    }
}

struct ExecutorFutureResult {
    plan_id: PlanId,
    result: ExecutionResult<ResponsePart>,
}
