use std::sync::Arc;

use futures_locks::Mutex;
use futures_util::{future::BoxFuture, stream::FuturesUnordered, StreamExt};

use super::{Executor, ExecutorContext, ExecutorError, ExecutorInput, ExecutorOutput};
use crate::{
    plan::{ExecutionPlansTracker, OperationPlan, PlanId},
    response::{GraphqlErrors, Response, ResponseData},
    Engine,
};

pub struct ExecutorCoordinator<'eng, 'op> {
    engine: &'eng Engine,
    plan: &'op OperationPlan,
    tracker: ExecutionPlansTracker,
    data: Arc<Mutex<ResponseData>>,
    errors: GraphqlErrors,
}

impl<'eng, 'op> ExecutorCoordinator<'eng, 'op> {
    pub fn new(engine: &'eng Engine, plan: &'op OperationPlan) -> Self {
        let tracker = plan.execution_plans.build_tracker();
        Self {
            engine,
            plan,
            data: Arc::new(Mutex::new(ResponseData::new(
                plan.operation.root_object_id,
                plan.operation.strings.clone(),
            ))),
            tracker,
            errors: GraphqlErrors::default(),
        }
    }

    pub async fn execute(&mut self) {
        let mut futures = FuturesUnordered::<BoxFuture<'_, CoordinationTask>>::new();
        for plan_id in self.tracker.all_without_dependencies() {
            futures.push(Box::pin(async move { CoordinationTask::PlanStart(plan_id) }));
        }
        while let Some(task) = futures.next().await {
            match task {
                CoordinationTask::PlanStart(plan_id) => {
                    let plan = &self.plan.execution_plans[plan_id];
                    let output = super::ExecutorOutput {
                        data: Arc::clone(&self.data),
                        errors: GraphqlErrors::default(),
                    };
                    if let Some(response_object_roots) =
                        self.data
                            .lock()
                            .await
                            .read_objects(&self.engine.schema, &plan.root.path, &plan.input)
                    {
                        let resolver = &self.engine.schema[plan.resolver_id];
                        let ctx = ExecutorContext {
                            engine: self.engine,
                            plan: self.plan,
                            plan_id,
                        };
                        let input = ExecutorInput {
                            root_response_objects: response_object_roots,
                        };
                        match Executor::build(ctx.clone(), resolver, input) {
                            Ok(executor) => {
                                futures.push(Box::pin(async move {
                                    let mut output = output;
                                    let error = executor.execute(ctx, &mut output).await.err();
                                    CoordinationTask::PlanEnd { plan_id, output, error }
                                }));
                            }
                            Err(err) => {
                                futures.push(Box::pin(async move {
                                    CoordinationTask::PlanEnd {
                                        output,
                                        error: Some(err),
                                        plan_id,
                                    }
                                }));
                            }
                        }
                    } else {
                        futures.push(Box::pin(async move {
                            CoordinationTask::PlanEnd {
                                output,
                                error: None,
                                plan_id,
                            }
                        }));
                    }
                }
                CoordinationTask::PlanEnd { plan_id, output, error } => {
                    // Creating a Graphql error if the executor failed
                    self.errors.push_errors(output.errors);
                    if let Some(error) = error {
                        let ctx = ExecutorContext {
                            engine: self.engine,
                            plan: self.plan,
                            plan_id,
                        };
                        let locations = ctx
                            .default_walk_selection_set()
                            .into_iter()
                            .map(|selection| selection.location())
                            .collect();
                        self.errors.add_error(vec![], error.to_string(), locations);
                    }
                    // start the execution of any children for which all parents have finised.
                    for executable_plan_id in self.tracker.next_executable(plan_id) {
                        futures.push(Box::pin(async move { CoordinationTask::PlanStart(executable_plan_id) }));
                    }
                    if self.tracker.are_all_executed() {
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
        let data = Arc::try_unwrap(self.data).unwrap().try_unwrap().unwrap();
        Response {
            data: Some(data.into_serializable(
                Arc::clone(&self.engine.schema),
                self.plan.final_read_selection_set.clone(),
            )),
            errors: self.errors.into(),
        }
    }
}

enum CoordinationTask {
    PlanStart(PlanId),
    PlanEnd {
        plan_id: PlanId,
        output: ExecutorOutput,
        error: Option<ExecutorError>,
    },
}
