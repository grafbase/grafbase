use std::collections::BTreeMap;

use async_runtime::make_send_on_wasm;
use engine::RequestHeaders;
use engine_parser::types::OperationType;
use futures_util::{
    future::BoxFuture,
    stream::{BoxStream, FuturesUnordered},
    Future, FutureExt, SinkExt, StreamExt,
};

use crate::{
    execution::{ExecutionContext, Variables},
    plan::{Plan, PlanBoundary, Planner, PlanningError},
    request::Operation,
    response::{ExecutionMetadata, ExecutorOutput, GraphqlError, Response, ResponseBoundaryItem, ResponseBuilder},
    sources::{Executor, ExecutorResult, ResolverInput, SubscriptionExecutor, SubscriptionResolverInput},
    Engine,
};

pub type ResponseReceiver = futures::channel::mpsc::Receiver<Response>;
pub type ResponseSender = futures::channel::mpsc::Sender<Response>;

pub struct ExecutorCoordinator<'ctx> {
    engine: &'ctx Engine,
    operation: Operation,
    variables: Variables,
    request_headers: RequestHeaders,
}

impl<'ctx> ExecutorCoordinator<'ctx> {
    pub fn new(
        engine: &'ctx Engine,
        operation: Operation,
        variables: Variables,
        request_headers: RequestHeaders,
    ) -> Self {
        Self {
            engine,
            operation,
            variables,
            request_headers,
        }
    }

    pub fn operation_type(&self) -> OperationType {
        self.operation.ty
    }

    pub async fn execute(self) -> Response {
        let mut planner = Planner::new(&self.engine.schema, &self.operation);
        let mut response = ResponseBuilder::new(self.operation.root_object_id);

        // Mutation root fields need to be executed sequentially. So we're tracking for each
        // executor whether it was for one and if so execute the next executor in the queue.
        // Keeping the queue outside of the FuturesUnordered also ensures the future is static
        // which wasm target somehow required. (not entirely sure why though)
        let mut mutation_root_fields_executors = vec![];
        let response_boundary = vec![response
            .root_response_boundary()
            .expect("a fresh response should always have a root")];

        let initial_executors = match planner.generate_root_plan_boundary() {
            Ok(boundary) => self.generate_executors(vec![(boundary, response_boundary)], &mut planner, &mut response),
            Err(error) => {
                response.push_error(error.into_graphql_error());

                vec![]
            }
        };

        let mut futures = ExecutorFutureSet::new();

        match self.operation.ty {
            OperationType::Query => {
                for executor in initial_executors {
                    futures.execute(executor);
                }
            }
            OperationType::Mutation => {
                // Reverse the initial_executors so we can pop them in the order they should run
                mutation_root_fields_executors = initial_executors;
                mutation_root_fields_executors.reverse();

                if let Some(executor) = mutation_root_fields_executors.pop() {
                    futures.execute_mutation_root(executor);
                }
            }
            OperationType::Subscription => {
                unreachable!("execute shouldnt be called for subscriptions")
            }
        }

        self.execute_once(futures, response, mutation_root_fields_executors, &mut planner)
            .await
    }

    pub async fn execute_subscription(self, mut responses: ResponseSender) {
        assert!(matches!(self.operation.ty, OperationType::Subscription));

        let mut planner = Planner::new(&self.engine.schema, &self.operation);

        let mut stream = match self.build_subscription_stream(&mut planner).await {
            Ok(stream) => stream,
            Err(error) => {
                responses
                    .send(
                        ResponseBuilder::new(self.operation.root_object_id)
                            .with_error(error)
                            .build(
                                self.engine.schema.clone(),
                                self.operation.response_keys.clone(),
                                ExecutionMetadata::build(&self.operation),
                            ),
                    )
                    .await
                    .ok();
                return;
            }
        };

        while let Some((response, output)) = stream.next().await {
            let mut futures = ExecutorFutureSet::new();
            futures.push(async move { ExecutorFutResult::from(Ok(output)) });

            let response = self.execute_once(futures, response, vec![], &mut planner).await;

            if responses.send(response).await.is_err() {
                return;
            }
        }
    }

    /// Runs a single execution to completion, returning its response
    async fn execute_once(
        &'ctx self,
        mut futures: ExecutorFutureSet<'ctx>,
        mut response: ResponseBuilder,
        mut mutation_root_fields_executors: Vec<Executor<'ctx>>,
        planner: &mut Planner<'ctx>,
    ) -> Response {
        while let Some(ExecutorFutResult {
            result,
            is_mutation_root_field,
        }) = futures.next().await
        {
            match result {
                Ok(output) => {
                    // Ingesting data first to propagate errors.
                    let boundaries = response.ingest(output);

                    // Hack to ensure we don't execute any subsequent mutation root fields if a
                    // previous one failed and the error propagated up to the root `data` field.
                    if response.root_response_object_id().is_some() {
                        if is_mutation_root_field {
                            if let Some(executor) = mutation_root_fields_executors.pop() {
                                futures.execute_mutation_root(executor);
                            }
                        }
                        let executors = self.generate_executors(boundaries, planner, &mut response);
                        for executor in executors {
                            futures.execute(executor);
                        }
                    }
                }
                Err(err) => {
                    response.push_error(err);
                }
            }
        }

        response.build(
            self.engine.schema.clone(),
            self.operation.response_keys.clone(),
            ExecutionMetadata::build(&self.operation),
        )
    }

    async fn build_subscription_stream<'a>(
        &'a self,
        planner: &mut Planner<'a>,
    ) -> Result<BoxStream<'a, (ResponseBuilder, ExecutorOutput)>, GraphqlError> {
        let plan = planner
            .generate_root_plan_boundary()
            .and_then(|boundary| planner.generate_subscription_plan(boundary))
            .map_err(PlanningError::into_graphql_error)?;

        let executor = self.subscription_executor_from_plan(plan)?;

        Ok(executor.execute().await?)
    }

    fn generate_executors(
        &self,
        boundaries: Vec<(PlanBoundary, Vec<ResponseBoundaryItem>)>,
        planner: &mut Planner<'ctx>,
        response: &mut ResponseBuilder, // mutable_bits: &mut TheMutableBits<'ctx>,
    ) -> Vec<Executor<'_>> {
        // Ordering of the executors MUST match the plan boundary order for mutation root
        let mut executors = vec![];

        for (plan_boundary, response_boundaries) in boundaries {
            let plans = match planner.generate_plans(plan_boundary, &response_boundaries) {
                Ok(plans) => plans,
                Err(error) => {
                    response.push_error(error.into_graphql_error());
                    continue;
                }
            };

            for plan in plans {
                match self.executor_from_plan(plan, response) {
                    Ok(executor) => executors.push(executor),
                    Err(error) => response.push_error(error),
                }
            }
        }

        executors
    }

    fn executor_from_plan<'a>(&'a self, plan: Plan, response: &mut ResponseBuilder) -> ExecutorResult<Executor<'a>> {
        let resolver = self.engine.schema.walker().walk(plan.resolver_id);
        let schema = self.engine.schema.walker_with(resolver.names());
        let output = response.new_output(plan.boundaries);
        // Ensuring that all walkers the executors has access to have a consistent
        // `Names`.
        let resolver = schema.walk(plan.resolver_id);
        Executor::build(
            resolver,
            plan.output.entity_type,
            ResolverInput {
                ctx: ExecutionContext {
                    engine: self.engine,
                    variables: &self.variables,
                    walker: self.operation.walker_with(schema),
                    request_headers: &self.request_headers,
                },
                boundary_objects_view: response
                    .read(schema, plan.input.expect("all but the subscription plan to have input")),
                plan_id: plan.id,
                plan_output: plan.output,
                output,
            },
        )
    }

    fn subscription_executor_from_plan(&self, plan: Plan) -> ExecutorResult<SubscriptionExecutor<'_>> {
        let resolver = self.engine.schema.walker().walk(plan.resolver_id);
        let schema = self.engine.schema.walker_with(resolver.names());

        // Ensuring that all walkers the executors has access to have a consistent
        // `Names`.
        let resolver = schema.walk(plan.resolver_id);

        SubscriptionExecutor::build(
            resolver,
            plan.output.entity_type,
            SubscriptionResolverInput {
                ctx: ExecutionContext {
                    engine: self.engine,
                    variables: &self.variables,
                    walker: self.operation.walker_with(schema),
                    request_headers: &self.request_headers,
                },
                plan_id: plan.id,
                plan_output: plan.output,
                plan_boundaries: plan.boundaries,
            },
        )
    }
}

impl PlanningError {
    fn into_graphql_error(self) -> GraphqlError {
        GraphqlError {
            message: self.to_string(),
            locations: vec![],
            path: None,
            extensions: BTreeMap::from([("queryPath".into(), serde_json::Value::Array(vec![]))]),
        }
    }
}

pub struct ExecutorFutureSet<'a>(FuturesUnordered<BoxFuture<'a, ExecutorFutResult>>);

impl<'a> ExecutorFutureSet<'a> {
    fn new() -> Self {
        ExecutorFutureSet(FuturesUnordered::new())
    }

    fn execute(&mut self, executor: Executor<'a>) {
        self.push(make_send_on_wasm(executor.execute().map(ExecutorFutResult::from)))
    }

    fn execute_mutation_root(&mut self, executor: Executor<'a>) {
        self.push(make_send_on_wasm(async move {
            ExecutorFutResult {
                result: executor.execute().await,
                is_mutation_root_field: true,
            }
        }))
    }

    fn push(&mut self, fut: impl Future<Output = ExecutorFutResult> + Send + 'a) {
        self.0.push(Box::pin(fut));
    }

    async fn next(&mut self) -> Option<ExecutorFutResult> {
        self.0.next().await
    }
}

struct ExecutorFutResult {
    result: ExecutorResult<ExecutorOutput>,
    is_mutation_root_field: bool,
}

impl From<ExecutorResult<ExecutorOutput>> for ExecutorFutResult {
    fn from(result: ExecutorResult<ExecutorOutput>) -> Self {
        Self {
            result,
            is_mutation_root_field: false,
        }
    }
}
