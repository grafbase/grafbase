use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

use async_runtime::make_send_on_wasm;
use engine::RequestHeaders;
use engine_parser::types::OperationType;
use futures_util::{future::BoxFuture, stream::FuturesUnordered, Future, FutureExt, Stream, StreamExt};

use crate::{
    execution::{ExecutionContext, Variables},
    plan::{Plan, PlanBoundary, Planner, PlanningError},
    request::{Operation, QueryPath},
    response::{
        ExecutionMetadata, ExecutorOutput, GraphqlError, Response, ResponseBoundaryItem, ResponseBuilder, ResponsePath,
    },
    sources::{Executor, ExecutorResult, ResolverInput, SubscriptionExecutor},
    Engine,
};

pub struct ExecutorCoordinator<'ctx> {
    engine: &'ctx Engine,
    operation: &'ctx Operation,
    planner: Planner<'ctx>,
    variables: &'ctx Variables<'ctx>,
    request_headers: &'ctx RequestHeaders,
}

impl<'ctx> ExecutorCoordinator<'ctx> {
    pub fn new(
        engine: &'ctx Engine,
        operation: &'ctx Operation,
        variables: &'ctx Variables<'ctx>,
        request_headers: &'ctx RequestHeaders,
    ) -> Self {
        Self {
            engine,
            operation,
            planner: Planner::new(&engine.schema, operation),
            variables,
            request_headers,
        }
    }

    pub async fn execute(mut self) -> Response {
        let mut response = ResponseBuilder::new(&self.operation);

        // Mutation root fields need to be executed sequentially. So we're tracking for each
        // executor whether it was for one and if so execute the next executor in the queue.
        // Keeping the queue outside of the FuturesUnordered also ensures the future is static
        // which wasm target somehow required. (not entirely sure why though)
        let mut mutation_root_fields_executors = VecDeque::<Executor<'ctx>>::new();
        let response_boundary = vec![response
            .root_response_boundary()
            .expect("a fresh response should always have a root")];

        let plan_boundary = match self.planner.generate_root_plan_boundary() {
            Ok(boundary) => boundary,
            Err(err) => {
                self.push_planning_error(QueryPath::default(), err, &mut response);
                todo!("return somehow");
            }
        };

        let mut futures = ExecutorFutureSet::new();

        let initial_exectors = self.generate_executors(vec![(plan_boundary, response_boundary)], &mut response);

        match self.operation.ty {
            OperationType::Query => {
                for executor in initial_exectors {
                    futures.execute(executor);
                }
            }
            OperationType::Mutation => {
                // TODO: Maybe just do a reversed vec for these?
                mutation_root_fields_executors = VecDeque::from(initial_exectors);

                if let Some(executor) = mutation_root_fields_executors.pop_front() {
                    futures.execute_mutation_root(executor);
                }
            }
            OperationType::Subscription => {
                unreachable!("execute shouldnt be called for subscriptions")
            }
        }

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
                            if let Some(executor) = mutation_root_fields_executors.pop_front() {
                                futures.execute_mutation_root(executor);
                            }
                        }
                        let executors = self.generate_executors(boundaries, &mut response);
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
            ExecutionMetadata::build(self.operation),
        )
    }

    pub fn execute_subscription(&mut self) -> impl Stream<Item = Response> + 'ctx {
        // TODO: Decide if this is even the best place to put this or if we need some
        // new kind of cordinator?  Who knows
        assert!(matches!(self.operation.ty, OperationType::Subscription));

        let response_boundary = vec![response
            .root_response_boundary()
            .expect("a fresh response should always have a root")];

        // TODO: wonder if there's scope to combine generate_root_plan_boundary & plans_from_boundary
        // Think we always call them together?
        let plan = match self.planner.generate_root_plan_boundary() {
            Ok(boundary) => {
                let mut plans = self.plans_from_boundary((boundary, response_boundary));
                // TODO: This probably shouldn't be an assert but yolo for now
                assert!(plans.len() == 1);
                plans.pop().expect("TODO: make this less hacky")
            }
            Err(err) => {
                // self.push_planning_error(QueryPath::default(), err);
                todo!("return an error somehow")
            }
        };

        let Some(executor) = self.subscription_executor_from_plan(plan, todo!()) else {
            todo!("return an error somehow")
        };

        let coordinator = Arc::new(self);
        executor.execute().then({
            move |(mut response, output)| {
                let coordinator = Arc::clone(&coordinator);
                async move {
                    let boundaries = response.ingest(output);
                    let mut futures = ExecutorFutureSet::new();
                    for executor in coordinator.generate_executors(boundaries, &mut response) {
                        futures.execute(executor);
                    }

                    while let Some(ExecutorFutResult { result, .. }) = futures.next().await {
                        let output = match result {
                            Ok(output) => output,
                            Err(error) => {
                                response.push_error(error);
                                continue;
                            }
                        };

                        let boundaries = response.ingest(output);
                        if response.root_response_object_id().is_some() {
                            // TODO: THis needs to somehow use the actual response not self.response
                            let executors = coordinator.generate_executors(boundaries, &mut response);
                            for executor in executors {
                                futures.execute(executor);
                            }
                        }
                    }

                    response.build(
                        coordinator.engine.schema.clone(),
                        coordinator.operation.response_keys.clone(),
                        ExecutionMetadata::build(coordinator.operation),
                    )
                }
            }
        })
    }

    fn generate_executors(
        &mut self,
        boundaries: Vec<(PlanBoundary, Vec<ResponseBoundaryItem>)>,
        response: &mut ResponseBuilder,
    ) -> Vec<Executor<'ctx>> {
        // Ordering of the executors MUST match the plan boundary order for mutation root
        let mut executors = vec![];

        for boundary in boundaries {
            for plan in self.plans_from_boundary(boundary) {
                executors.extend(self.executor_from_plan(plan, response))
            }
        }

        executors
    }

    fn plans_from_boundary(
        &mut self,
        (plan_boundary, response_boundary): (PlanBoundary, Vec<ResponseBoundaryItem>),
    ) -> Vec<Plan> {
        let query_path = plan_boundary.query_path.clone();
        match self.planner.generate_plans(plan_boundary, &response_boundary) {
            Ok(plans) => plans,

            Err(err) => {
                todo!("self.push_planning_error(query_path, err)");
                vec![]
            }
        }
    }

    // This function requires:
    // - schema from self.engine
    // - the current response
    // - An execution context, mostly generatable from self
    fn executor_from_plan(&mut self, plan: Plan, response: &mut ResponseBuilder) -> Option<Executor<'ctx>> {
        let resolver = self.engine.schema.walker().walk(plan.resolver_id);
        let schema = self.engine.schema.walker_with(resolver.names());
        let output = response.new_output(plan.boundaries);
        // Ensuring that all walkers the executors has access to have a consistent
        // `Names`.
        let resolver = schema.walk(plan.resolver_id);
        let result = Executor::build(
            resolver,
            plan.output.entity_type,
            ResolverInput {
                ctx: ExecutionContext::<'ctx> {
                    engine: self.engine,
                    variables: self.variables,
                    walker: self.operation.walker_with(schema, ()),
                    request_headers: self.request_headers,
                },
                boundary_objects_view: response.read(schema, plan.input),
                plan_id: plan.id,
                plan_output: plan.output,
                output,
            },
        );
        match result {
            Ok(executor) => Some(executor),
            Err(err) => {
                response.push_error(err);
                None
            }
        }
    }

    fn subscription_executor_from_plan(
        &mut self,
        plan: Plan,
        response: &mut ResponseBuilder,
    ) -> Option<SubscriptionExecutor<'ctx>> {
        let resolver = self.engine.schema.walker().walk(plan.resolver_id);
        let schema = self.engine.schema.walker_with(resolver.names());
        let output = response.new_output(plan.boundaries);
        // Ensuring that all walkers the executors has access to have a consistent
        // `Names`.
        let resolver = schema.walk(plan.resolver_id);
        let result = SubscriptionExecutor::build(
            resolver,
            plan.output.entity_type,
            todo!(), // ResolverInput {
                     //     ctx: ExecutionContext::<'ctx> {
                     //         engine: self.engine,
                     //         variables: self.variables,
                     //         walker: self.operation.walker_with(schema, ()),
                     //         request_headers: self.request_headers,
                     //     },
                     //     boundary_objects_view: self.response.read(schema, plan.input),
                     //     plan_id: plan.id,
                     //     plan_output: plan.output,
                     //     output,
                     // },
        );
        match result {
            Ok(executor) => Some(executor),
            Err(err) => {
                response.push_error(err);
                None
            }
        }
    }

    fn push_planning_error(&mut self, query_path: QueryPath, err: PlanningError, response: &mut ResponseBuilder) {
        response.push_error(GraphqlError {
            message: err.to_string(),
            locations: vec![],
            path: None,
            extensions: BTreeMap::from([(
                "queryPath".into(),
                serde_json::Value::Array(
                    query_path
                        .into_iter()
                        .map(|key| self.operation.response_keys[*key].into())
                        .collect(),
                ),
            )]),
        })
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

    pub async fn next(&mut self) -> Option<ExecutorFutResult> {
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
