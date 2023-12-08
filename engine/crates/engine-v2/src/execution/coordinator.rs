use async_runtime::make_send_on_wasm;
use futures_util::{future::BoxFuture, stream::FuturesUnordered, StreamExt};

use crate::{
    execution::{ExecutionContext, Variables},
    plan::{PlanBoundary, Planner},
    request::Operation,
    response::{ExecutionMetadata, ExecutorOutput, Response, ResponseBoundaryItem, ResponseBuilder, ResponsePath},
    sources::{Executor, ExecutorResult, ResolverInput},
    Engine,
};

pub struct ExecutorCoordinator<'ctx> {
    engine: &'ctx Engine,
    operation: &'ctx Operation,
    planner: Planner<'ctx>,
    response: ResponseBuilder,
    variables: &'ctx Variables<'ctx>,
}

impl<'ctx> ExecutorCoordinator<'ctx> {
    pub fn new(engine: &'ctx Engine, operation: &'ctx Operation, variables: &'ctx Variables<'ctx>) -> Self {
        Self {
            engine,
            operation,
            planner: Planner::new(&engine.schema, operation),
            response: ResponseBuilder::new(operation),
            variables,
        }
    }

    pub async fn execute(&mut self) {
        let mut futures = FuturesUnordered::<BoxFuture<'_, ExecutorResult<ExecutorOutput>>>::new();
        match self.planner.generate_initial_boundary() {
            Ok(plans) => {
                let boundary = vec![ResponseBoundaryItem {
                    response_object_id: self
                        .response
                        .root_response_object_id()
                        .expect("No errors could have propagated to root yet."),
                    response_path: ResponsePath::default(),
                    object_id: self.operation.root_object_id,
                }];

                let executors = self.generate_executors(vec![(plans, boundary)]);
                for executor in executors {
                    futures.push(Box::pin(make_send_on_wasm(executor.execute())));
                }
            }
            Err(err) => {
                self.response.push_error(err);
            }
        }
        while let Some(result) = futures.next().await {
            match result {
                Ok(output) => {
                    let boundaries = self.response.ingest(output);
                    let executors = self.generate_executors(boundaries);
                    for executor in executors {
                        futures.push(Box::pin(make_send_on_wasm(executor.execute())));
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
        let mut executors = vec![];
        for (boundary, response_objects) in boundaries {
            match self.planner.generate_plans(boundary, &response_objects) {
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
                                    walker: self.operation.walker_with(schema, ()),
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
                    self.response.push_error(err);
                }
            }
        }
        executors
    }

    // ugly... should be sent back through a stream to support defer.
    pub fn into_response(self) -> Response {
        self.response
            .build(self.engine.schema.clone(), ExecutionMetadata::build(self.operation))
    }
}
