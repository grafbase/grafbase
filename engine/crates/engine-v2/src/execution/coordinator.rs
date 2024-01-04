use std::collections::{BTreeMap, VecDeque};

use async_runtime::make_send_on_wasm;
use engine::RequestHeaders;
use engine_parser::types::OperationType;
use futures_util::{future::BoxFuture, stream::FuturesUnordered, FutureExt, StreamExt};

use crate::{
    execution::{ExecutionContext, Variables},
    plan::{PlanBoundary, Planner, PlanningError},
    request::{Operation, QueryPath},
    response::{
        ExecutionMetadata, ExecutorOutput, GraphqlError, Response, ResponseBoundaryItem, ResponseBuilder, ResponsePath,
    },
    sources::{Executor, ExecutorResult, ResolverInput},
    Engine,
};

pub struct ExecutorCoordinator<'ctx> {
    engine: &'ctx Engine,
    operation: &'ctx Operation,
    planner: Planner<'ctx>,
    response: ResponseBuilder,
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
            response: ResponseBuilder::new(operation),
            variables,
            request_headers,
        }
    }

    pub async fn execute(&mut self) {
        let mut futures = FuturesUnordered::<BoxFuture<'_, ExecutorFutResult>>::new();
        // Mutation root fields need to be executed sequentially. So we're tracking for each
        // executor whether it was for one and if so execute the next executor in the queue.
        // Keeping the queue outside of the FuturesUnordered also ensures the future is static
        // which wasm target somehow required. (not entirely sure why though)
        let mut mutation_root_fields_executors = VecDeque::<Executor<'ctx>>::new();
        let response_boundary = vec![ResponseBoundaryItem {
            response_object_id: self
                .response
                .root_response_object_id()
                .expect("No errors could have propagated to root yet."),
            response_path: ResponsePath::default(),
            object_id: self.operation.root_object_id,
        }];
        match self.operation.ty {
            OperationType::Query => match self.planner.generate_root_plan_boundary() {
                Ok(plan_boundary) => {
                    for executor in self.generate_executors(vec![(plan_boundary, response_boundary)]) {
                        futures.push(Box::pin(make_send_on_wasm(
                            executor.execute().map(ExecutorFutResult::from),
                        )));
                    }
                }
                Err(err) => {
                    self.push_planning_error(QueryPath::default(), err);
                }
            },
            OperationType::Mutation => match self.planner.generate_root_plan_boundary() {
                Ok(plan_boundary) => {
                    mutation_root_fields_executors =
                        VecDeque::from(self.generate_executors(vec![(plan_boundary, response_boundary)]));
                    if let Some(executor) = mutation_root_fields_executors.pop_front() {
                        futures.push(Box::pin(make_send_on_wasm(async move {
                            ExecutorFutResult {
                                result: executor.execute().await,
                                is_mutation_root_field: true,
                            }
                        })));
                    }
                }
                Err(err) => {
                    self.push_planning_error(QueryPath::default(), err);
                }
            },
            OperationType::Subscription => unimplemented!(),
        }
        while let Some(ExecutorFutResult {
            result,
            is_mutation_root_field,
        }) = futures.next().await
        {
            match result {
                Ok(output) => {
                    // Ingesting data first to propagate errors.
                    let boundaries = self.response.ingest(output);

                    // Hack to ensure we don't execute any subsequent mutation root fields if a
                    // previous one failed and the error propagated up to the root `data` field.
                    if self.response.root_response_object_id().is_some() {
                        if is_mutation_root_field {
                            if let Some(executor) = mutation_root_fields_executors.pop_front() {
                                futures.push(Box::pin(make_send_on_wasm(async move {
                                    ExecutorFutResult {
                                        result: executor.execute().await,
                                        is_mutation_root_field: true,
                                    }
                                })));
                            }
                        }
                        let executors = self.generate_executors(boundaries);
                        for executor in executors {
                            futures.push(Box::pin(make_send_on_wasm(
                                executor.execute().map(ExecutorFutResult::from),
                            )));
                        }
                    }
                }
                Err(err) => {
                    self.response.push_error(err);
                }
            }
        }
    }

    fn generate_executors(
        &mut self,
        boundaries: Vec<(PlanBoundary, Vec<ResponseBoundaryItem>)>,
    ) -> Vec<Executor<'ctx>> {
        // Ordering of the executors MUST match the plan boundary order for mutation root fields.
        let mut executors = vec![];
        for (plan_boundary, response_boundary) in boundaries {
            let query_path = plan_boundary.query_path.clone();
            match self.planner.generate_plans(plan_boundary, &response_boundary) {
                Ok(plans) => {
                    for plan in plans {
                        let resolver = self.engine.schema.walker().walk(plan.resolver_id);
                        let schema = self.engine.schema.walker_with(resolver.names());
                        let output = self.response.new_output(plan.boundaries);
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
                                    walker: self.operation.walker_with(schema),
                                    request_headers: self.request_headers,
                                },
                                boundary_objects_view: self.response.read(schema, plan.input),
                                plan_id: plan.id,
                                plan_output: plan.output,
                                output,
                            },
                        );
                        match result {
                            Ok(executor) => executors.push(executor),
                            Err(err) => {
                                self.response.push_error(err);
                            }
                        }
                    }
                }
                Err(err) => {
                    self.push_planning_error(query_path, err);
                }
            }
        }
        executors
    }

    fn push_planning_error(&mut self, query_path: QueryPath, err: PlanningError) {
        self.response.push_error(GraphqlError {
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

    // ugly... should be sent back through a stream to support defer.
    pub fn into_response(self) -> Response {
        self.response.build(
            self.engine.schema.clone(),
            self.operation.response_keys.clone(),
            ExecutionMetadata::build(self.operation),
        )
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
