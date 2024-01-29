use std::collections::HashMap;

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
    plan::{Plan, PlanBoundary, PlanId, Planner},
    request::Operation,
    response::{ExecutionMetadata, ExecutorOutput, GraphqlError, Response, ResponseBoundaryItem, ResponseBuilder},
    sources::{Executor, ExecutorResult, ResolverInput, SubscriptionExecutor, SubscriptionResolverInput},
    Engine,
};

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
        assert!(
            !matches!(self.operation.ty, OperationType::Subscription),
            "execute shouldn't be called for subscriptions"
        );

        let mut planner = Planner::new(&self.engine.schema, &self.operation);
        let mut response = ResponseBuilder::new(self.operation.root_object_id);

        let response_boundary = vec![response
            .root_response_boundary()
            .expect("a fresh response should always have a root")];

        let mut plans_with_dependencies = PlansWithDependencies::default();
        let executors = match planner.generate_root_plan_boundary() {
            Ok(boundary) => self.generate_executors(
                &mut planner,
                &mut plans_with_dependencies,
                &mut response,
                vec![(boundary, response_boundary)],
            ),
            Err(error) => {
                response.push_error(error);
                Vec::with_capacity(0)
            }
        };

        let mut futures = ExecutorFutureSet::new();
        for executor in executors {
            futures.execute(executor);
        }
        self.execute_once(futures, response, &mut planner, plans_with_dependencies)
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
            futures.push(async move {
                ExecutorFutResult {
                    result: Ok(output),
                    // Hack, we just know that the subscription plan is necessarily the first and
                    // there are no sibling plans anyway. So doesn't matter for now.
                    plan_id: PlanId::from(0),
                }
            });

            let response = self
                .execute_once(futures, response, &mut planner, PlansWithDependencies::default())
                .await;

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
        planner: &mut Planner<'ctx>,
        mut plans_with_dependencies: PlansWithDependencies,
    ) -> Response {
        while let Some(ExecutorFutResult { result, plan_id }) = futures.next().await {
            let output = match result {
                Ok(output) => output,
                Err(err) => {
                    response.push_error(err);
                    continue;
                }
            };

            // Ingesting data first to propagate errors and next plans likely rely on it
            let boundaries = response.ingest(output);

            // Hack to ensure we don't execute any subsequent mutation root fields if a
            // previous one failed and the error propagated up to the root `data` field.
            if response.root_response_object_id().is_none() {
                continue;
            }

            for plan in plans_with_dependencies.get_executable_plans(plan_id) {
                match self.executor_from_plan(plan, &mut response) {
                    Ok(executor) => futures.execute(executor),
                    Err(error) => response.push_error(error),
                }
            }

            let executors = self.generate_executors(planner, &mut plans_with_dependencies, &mut response, boundaries);

            for executor in executors {
                futures.execute(executor);
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
            .and_then(|boundary| planner.generate_subscription_plan(boundary))?;

        let executor = self.subscription_executor_from_plan(plan)?;

        Ok(executor.execute().await?)
    }

    fn generate_executors(
        &self,
        planner: &mut Planner<'ctx>,
        plans_with_dependencies: &mut PlansWithDependencies,
        response: &mut ResponseBuilder, // mutable_bits: &mut TheMutableBits<'ctx>,
        boundaries: Vec<(PlanBoundary, Vec<ResponseBoundaryItem>)>,
    ) -> Vec<Executor<'_>> {
        // Ordering of the executors MUST match the plan boundary order for mutation root
        let mut executors = vec![];

        for (plan_boundary, response_boundaries) in boundaries {
            let plans = match planner.generate_plans(plan_boundary, &response_boundaries) {
                Ok(plans) => plans,
                Err(error) => {
                    response.push_error(error);
                    continue;
                }
            };

            for plan in plans {
                if plan.sibling_dependencies.is_empty() {
                    match self.executor_from_plan(plan, response) {
                        Ok(executor) => executors.push(executor),
                        Err(error) => response.push_error(error),
                    }
                } else {
                    plans_with_dependencies.push(plan);
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

pub struct ExecutorFutureSet<'a>(FuturesUnordered<BoxFuture<'a, ExecutorFutResult>>);

impl<'a> ExecutorFutureSet<'a> {
    fn new() -> Self {
        ExecutorFutureSet(FuturesUnordered::new())
    }

    fn execute(&mut self, executor: Executor<'a>) {
        let plan_id = executor.plan_id();
        self.push(make_send_on_wasm(
            executor
                .execute()
                .map(move |result| ExecutorFutResult { result, plan_id }),
        ))
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
    plan_id: PlanId,
}

#[derive(Default)]
struct PlansWithDependencies {
    dependency_to_dependents: HashMap<PlanId, Vec<PlanId>>,
    plans: HashMap<PlanId, Plan>,
}

impl PlansWithDependencies {
    fn push(&mut self, plan: Plan) {
        assert!(!plan.sibling_dependencies.is_empty());
        for dependency in &plan.sibling_dependencies {
            self.dependency_to_dependents
                .entry(*dependency)
                .or_default()
                .push(plan.id);
        }
        self.plans.insert(plan.id, plan);
    }

    fn get_executable_plans(&mut self, plan_id: PlanId) -> Vec<Plan> {
        let mut executable_plans = vec![];
        for dependent in self.dependency_to_dependents.remove(&plan_id).unwrap_or_default() {
            let executable = self
                .plans
                .get_mut(&dependent)
                .map(|plan| {
                    plan.sibling_dependencies.remove(&plan_id);
                    plan.sibling_dependencies.is_empty()
                })
                .unwrap_or_default();
            if executable {
                executable_plans.push(self.plans.remove(&dependent).unwrap());
            }
        }

        executable_plans
    }
}
