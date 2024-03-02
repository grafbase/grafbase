use std::{borrow::Cow, sync::Arc, time::Duration};

use async_runtime::make_send_on_wasm;
use engine::HttpGraphqlResponse;
use engine_parser::types::OperationType;
use engine_v2_common::{OperationCacheControlCacheKey, ResponseCacheKey};
use futures_util::{
    future::BoxFuture,
    stream::{BoxStream, FuturesUnordered},
    Future, SinkExt, StreamExt,
};
#[cfg(feature = "tracing")]
use grafbase_tracing::span::{GqlRecorderSpanExt, GqlRequestAttributes, GqlResponseAttributes};
use headers::HeaderMapExt;
use runtime::{auth::AccessToken, cache::TaggedResponseContent};
#[cfg(feature = "tracing")]
use tracing::Span;

use crate::{
    execution::ExecutionContext,
    plan::{build_execution_metadata, OperationExecutionState, OperationPlan, PlanId},
    request::{OpInputValues, Operation},
    response::{GraphqlError, Response, ResponseBuilder, ResponsePart},
    sources::{Executor, ExecutorInput, SubscriptionExecutor, SubscriptionInput},
    Engine,
};

use super::ExecutionResult;

pub type ResponseSender = futures::channel::mpsc::Sender<Response>;

pub(crate) struct ExecutionCoordinator {
    engine: Arc<Engine>,
    headers: Arc<http::HeaderMap>,
    _access_token: Arc<AccessToken>,
    operation_plan: Arc<OperationPlan>,
    input_values: OpInputValues,
    response_cache_key: Option<ResponseCacheKey>,
    operation_cache_control_cache_key: Option<OperationCacheControlCacheKey>,
}

impl ExecutionCoordinator {
    pub fn new(
        engine: Arc<Engine>,
        headers: Arc<http::HeaderMap>,
        _access_token: Arc<AccessToken>,
        operation_plan: Arc<OperationPlan>,
        input_values: OpInputValues,
        response_cache_key: Option<ResponseCacheKey>,
        operation_cache_control_cache_key: Option<OperationCacheControlCacheKey>,
    ) -> Self {
        Self {
            engine,
            headers,
            _access_token,
            operation_plan,
            input_values,
            response_cache_key,
            operation_cache_control_cache_key,
        }
    }

    pub fn operation(&self) -> &Operation {
        &self.operation_plan
    }

    pub async fn cached_execute(self) -> HttpGraphqlResponse {
        if let Some(response_cache_key) = &self.response_cache_key {
            if let Some(operation_cache_control_cache_key) = &self.operation_cache_control_cache_key {
                self.background_cache_operation_cache_control(operation_cache_control_cache_key)
            }
            let cache = self.engine.env.cache.clone();
            let request_cache_control = self.headers.typed_get();
            let key = response_cache_key.to_string();
            let operation_cache_control = self.operation().cache_control.clone().unwrap();
            let result = cache
                .cached_execution(&key, request_cache_control, operation_cache_control, async move {
                    let response = self.execute().await;
                    let body = response.to_json_bytes()?;
                    if response.has_errors() {
                        Ok(TaggedResponseContent {
                            body,
                            cache_tags: Vec::new(),
                        })
                    } else {
                        Err(body)
                    }
                })
                .await;
            match result {
                Ok(cached_response) => cached_response.into(),
                Err(body) => HttpGraphqlResponse::from_json_bytes(body.into()),
            }
        } else {
            self.execute().await.into()
        }
    }

    pub async fn execute(self) -> Response {
        #[cfg(feature = "tracing")]
        let gql_span = Span::current();
        #[cfg(feature = "tracing")]
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

        #[cfg(feature = "tracing")]
        gql_span.record_gql_response(GqlResponseAttributes {
            has_errors: response.has_errors(),
        });

        response
    }

    pub async fn execute_subscription(self, mut responses: ResponseSender) {
        assert!(matches!(self.operation_plan.ty, OperationType::Subscription));

        #[cfg(feature = "tracing")]
        {
            let current_span = Span::current();
            current_span.record_gql_request(GqlRequestAttributes {
                operation_type: self.operation().ty.as_ref(),
                operation_name: self.operation().name.as_deref(),
            });
        }

        let mut state = self.operation_plan.new_execution_state();
        let subscription_plan_id = state.pop_subscription_plan_id();

        let mut stream = match self.build_subscription_stream(subscription_plan_id).await {
            Ok(stream) => stream,
            Err(error) => {
                responses
                    .send(
                        ResponseBuilder::new(self.operation_plan.root_object_id)
                            .with_error(error)
                            .build(
                                self.engine.schema.clone(),
                                self.operation_plan.clone(),
                                build_execution_metadata(&self.engine.schema, &self.operation_plan),
                            ),
                    )
                    .await
                    .ok();
                return;
            }
        };

        while let Some((response, output)) = stream.next().await {
            let mut futures = ExecutorFutureSet::new();
            futures.push(async move {
                ExecutorFutureResult {
                    result: Ok(output),
                    plan_id: subscription_plan_id,
                }
            });
            let response = OperationExecution {
                coordinator: &self,
                futures,
                state: state.clone(),
                response,
            }
            .execute()
            .await;

            if responses.send(response).await.is_err() {
                return;
            }
        }
    }

    async fn build_subscription_stream(
        &self,
        plan_id: PlanId,
    ) -> Result<BoxStream<'_, (ResponseBuilder, ResponsePart)>, GraphqlError> {
        let executor = self.build_subscription_executor(plan_id)?;
        Ok(executor.execute().await?)
    }

    fn build_subscription_executor(&self, plan_id: PlanId) -> ExecutionResult<SubscriptionExecutor<'_>> {
        let execution_plan = &self.operation_plan[plan_id];
        let plan = self
            .operation_plan
            .plan_walker(&self.engine.schema, plan_id, Some(&self.input_values));
        let input = SubscriptionInput {
            ctx: ExecutionContext {
                engine: self.engine.as_ref(),
                headers: &self.headers,
            },
            plan,
        };
        execution_plan.new_subscription_executor(input)
    }

    fn background_cache_operation_cache_control(
        &self,
        operation_cache_control_cache_key: &OperationCacheControlCacheKey,
    ) {
        let operation_cache_control = self
            .operation()
            .cache_control
            .clone()
            .expect("Cannot have a cache key if empty");
        let key = operation_cache_control_cache_key.to_string();
        let cache = self.engine.env.cache.clone();
        self.engine.env.async_runtime.spawn_faillible(async move {
            cache
                .put_json(
                    &key.to_string(),
                    &operation_cache_control,
                    Duration::from_secs(24 * 60 * 60),
                )
                .await
        });
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
            build_execution_metadata(&self.coordinator.engine.schema, &self.coordinator.operation_plan),
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
                headers: &self.coordinator.headers,
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
