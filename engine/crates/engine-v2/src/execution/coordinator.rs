use std::collections::HashMap;

use async_runtime::make_send_on_wasm;
use futures_util::{future::BoxFuture, stream::FuturesUnordered, StreamExt};

use super::ExecutionContext;
use crate::{
    execution::Variables,
    executor::{Executor, ExecutorError, ExecutorInput},
    plan::{ExecutionPlan, ExecutionPlans, PlanId},
    request::Operation,
    response::{GraphqlError, Response, ResponseBuilder, ResponsePartBuilder},
    Engine,
};

pub struct ExecutorCoordinator<'eng, 'op> {
    engine: &'eng Engine,
    operation: &'op Operation,
    plans: ExecutionPlans,
    response: ResponseBuilder,
    variables: Variables<'op>,
}

impl<'eng, 'op> ExecutorCoordinator<'eng, 'op> {
    pub fn new(engine: &'eng Engine, operation: &'op Operation, variables: Variables<'op>) -> Self {
        let plans = ExecutionPlans::initialize(engine, operation);
        Self {
            engine,
            operation,
            plans,
            response: ResponseBuilder::new(&engine.schema, operation),
            variables,
        }
    }

    pub async fn execute(&mut self) {
        let mut futures = FuturesUnordered::<BoxFuture<'_, CoordinationTask>>::new();
        for plan_id in self.plans.all_without_dependencies() {
            futures.push(Box::pin(make_send_on_wasm(async move {
                CoordinationTask::PlanStart(plan_id)
            })));
        }
        while let Some(task) = futures.next().await {
            match task {
                CoordinationTask::PlanStart(plan_id) => {
                    let plan = &self.plans[plan_id];
                    if let Some(response_object_roots) =
                        self.response
                            .read_objects(&self.engine.schema, &plan.root.path, &plan.input)
                    {
                        let resolver = &self.engine.schema[plan.resolver_id];
                        let input = ExecutorInput {
                            root_response_objects: response_object_roots,
                        };
                        let ctx = ExecutionContext {
                            engine: self.engine,
                            operation: self.operation,
                            names: self.engine.schema.as_ref(),
                            plan_id,
                            // FIXME: Is there a better way?
                            // The only purpose of transmute is to change the lifetime. The only
                            // alternative I could think of are Arc. But they're somewhat a pain to
                            // deal with as they're owned making everything else depend on the
                            // ExecutionContext own lifetime instead of the 'ctx one.
                            // SAFETY: We're never deleting plans during execution
                            // (lifetime of the futures using the ExecutionContext). And plans
                            // live inside a Vec<_> so they are also never moved.
                            plan: unsafe { std::mem::transmute::<&ExecutionPlan, &ExecutionPlan>(plan) },
                            variables: &self.variables,
                        };
                        match Executor::build(ctx.clone(), resolver, input) {
                            Ok(executor) => {
                                let mut data_part = self.response.new_part();
                                futures.push(Box::pin(make_send_on_wasm(async move {
                                    let executor_error = executor.execute(ctx, &mut data_part).await.err();
                                    CoordinationTask::PlanEnd {
                                        plan_id,
                                        data: Some(data_part),
                                        executor_error,
                                    }
                                })));
                            }
                            Err(err) => {
                                futures.push(Box::pin(make_send_on_wasm(async move {
                                    CoordinationTask::PlanEnd {
                                        executor_error: Some(err),
                                        data: None,
                                        plan_id,
                                    }
                                })));
                            }
                        }
                    } else {
                        futures.push(Box::pin(make_send_on_wasm(async move {
                            CoordinationTask::PlanEnd {
                                executor_error: None,
                                data: None,
                                plan_id,
                            }
                        })));
                    }
                }
                CoordinationTask::PlanEnd {
                    plan_id,
                    data,
                    executor_error,
                } => {
                    // if let Some(error) = error {
                    // let plan = &self.plans[plan_id];
                    // let ctx = ExecutionContext::new(
                    //     &self.engine,
                    //     &self.operation,
                    //     &plan.attribution,
                    //     plan.root.clone(),
                    //     &self.variables,
                    // );
                    //
                    // let locations = ctx
                    //     .default_walk_selection_set()
                    //     .into_iter()
                    //     .map(|selection| selection.location())
                    //     .collect();
                    // self.errors.add_error(vec![], error.to_string(), locations);
                    //     todo!()
                    // }
                    if let Some(data) = data {
                        self.response.ingest_part(data);
                    }
                    if let Some(error) = executor_error {
                        self.response.push_error(GraphqlError {
                            message: error.to_string(),
                            // TODO: fix locations (from plan) & path (from input?)
                            locations: vec![],
                            path: None,
                            extensions: HashMap::with_capacity(0),
                        });
                    }

                    // start the execution of any children for which all parents have finised.
                    for executable_plan_id in self.plans.next_executable(plan_id) {
                        futures.push(Box::pin(make_send_on_wasm(async move {
                            CoordinationTask::PlanStart(executable_plan_id)
                        })));
                    }
                    if self.plans.are_all_executed() {
                        // We should data back through a stream to later support additional data
                        // being sent like @defer or streaming
                        break;
                    }
                }
            }
        }
    }

    // ugly... should be sent back through a stream to support defer.
    pub fn into_response(self) -> Response {
        self.response.build(self.engine.schema.clone())
    }
}

enum CoordinationTask {
    PlanStart(PlanId),
    PlanEnd {
        plan_id: PlanId,
        data: Option<ResponsePartBuilder>,
        executor_error: Option<ExecutorError>,
    },
}
