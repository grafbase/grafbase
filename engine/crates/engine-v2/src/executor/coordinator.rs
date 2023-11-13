use std::sync::Arc;

use engine_parser::types::OperationType;
use futures_locks::Mutex;
use futures_util::StreamExt;

use super::{Executor, ExecutorRequest, ResponseProxy};
use crate::{
    plan::{ExecutionPlans, PlanId, RequestPlan},
    response::{Response, SelectionSet},
    Engine,
};

pub struct ExecutorCoordinator<'a> {
    engine: &'a Engine,
    operation_type: OperationType,
    operation_selection_set: SelectionSet,
    plans: ExecutionPlans,
    response: Arc<Mutex<Response>>,
}

impl<'a> ExecutorCoordinator<'a> {
    pub fn new(engine: &'a Engine, request_plan: RequestPlan) -> Self {
        Self {
            engine,
            operation_type: request_plan.operation_type,
            operation_selection_set: request_plan.operation_selection_set,
            response: Arc::new(Mutex::new(Response::new(
                engine.schema.clone(),
                request_plan.response_fields,
            ))),
            plans: request_plan.execution_plans,
        }
    }

    pub async fn execute(&mut self) -> serde_json::Value {
        let (sender, mut receiver) = futures_channel::mpsc::unbounded::<CoordinationTask>();
        for plan_id in self.plans.all_without_dependencies() {
            sender.unbounded_send(CoordinationTask::PlanStart(plan_id)).unwrap();
        }
        while let Some(task) = receiver.next().await {
            match task {
                CoordinationTask::PlanStart(plan_id) => {
                    let plan = &self.plans[plan_id];
                    if let Some(input) = self.response.lock().await.view(&plan.path, &plan.input) {
                        let resolver = &self.engine.schema[plan.resolver_id];
                        let execution_plan = Executor::build(
                            self.engine,
                            resolver,
                            ExecutorRequest {
                                operation_type: self.operation_type,
                                response_objects: input,
                                output: &plan.output,
                            },
                        );
                        let sender = sender.clone();
                        let proxy = ResponseProxy {
                            inner: self.response.clone(),
                        };
                        async_runtime::spawn(async move {
                            execution_plan.execute(proxy).await;
                            sender.unbounded_send(CoordinationTask::PlanEnd(plan_id)).unwrap();
                        });
                    } else {
                        sender.unbounded_send(CoordinationTask::PlanEnd(plan_id)).unwrap();
                    }
                }
                CoordinationTask::PlanEnd(plan_id) => {
                    // start the execution of any children for which all parents have finised.
                    for executable_plan_id in self.plans.finished(plan_id) {
                        sender
                            .unbounded_send(CoordinationTask::PlanStart(executable_plan_id))
                            .unwrap();
                    }
                    if self.plans.are_all_executed() {
                        // We should data back through a stream to later support additional data
                        // being sent like @defer or streaming
                        break;
                    }
                }
            }
        }

        // Response will be sent through a byte stream back. Client shouldn't be aware of wether we
        // defer / stream / etc.
        serde_json::Value::Null
    }
}

enum CoordinationTask {
    PlanStart(PlanId),
    PlanEnd(PlanId),
}
