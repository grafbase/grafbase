use std::sync::Arc;

use engine_parser::types::OperationType;
use futures_locks::Mutex;
use futures_util::{future::BoxFuture, stream::FuturesUnordered, StreamExt};

use super::{Executor, ExecutorContext, ExecutorError};
use crate::{
    plan::{ExecutionPlans, PlanId, PlannedOperation},
    request::OperationFields,
    response::{ReadSelectionSet, Response},
    Engine,
};

pub struct ExecutorCoordinator<'a> {
    engine: &'a Engine,
    operation_type: OperationType,
    operation_fields: OperationFields,
    final_read_selection_set: ReadSelectionSet,
    plans: ExecutionPlans,
    response: Arc<Mutex<Response>>,
}

impl<'a> ExecutorCoordinator<'a> {
    pub fn new(engine: &'a Engine, planned_operation: PlannedOperation) -> Self {
        Self {
            engine,
            operation_type: planned_operation.ty,
            final_read_selection_set: planned_operation.final_read_selection_set,
            operation_fields: planned_operation.fields,
            response: Arc::new(Mutex::new(Response::new(planned_operation.strings))),
            plans: planned_operation.plans,
        }
    }

    pub async fn execute(&mut self) -> serde_json::Value {
        let mut futures = FuturesUnordered::<BoxFuture<'static, CoordinationTask>>::new();
        for plan_id in self.plans.all_without_dependencies() {
            futures.push(Box::pin(async move { CoordinationTask::PlanStart(plan_id) }));
        }
        while let Some(task) = futures.next().await {
            match task {
                CoordinationTask::PlanStart(plan_id) => {
                    let plan = &self.plans[plan_id];
                    let response = self.response.lock().await;
                    if let Some(response_objects) = response.read_objects(&plan.path, &plan.input) {
                        let resolver = &self.engine.schema[plan.resolver_id];
                        match Executor::build(
                            self.engine,
                            resolver,
                            ExecutorContext {
                                operation_type: self.operation_type,
                                operation_fields: &self.operation_fields,
                                response: &response,
                                response_object_roots: response_objects,
                                selection_set: &plan.selection_set,
                            },
                        ) {
                            Ok(executor) => {
                                let response = Arc::clone(&self.response);
                                futures.push(Box::pin(async move {
                                    let result = executor.execute(response).await;
                                    CoordinationTask::PlanEnd { result, plan_id }
                                }));
                            }
                            Err(err) => {
                                futures.push(Box::pin(async move {
                                    CoordinationTask::PlanEnd {
                                        result: Err(err),
                                        plan_id,
                                    }
                                }));
                            }
                        }
                    } else {
                        futures.push(Box::pin(async move {
                            CoordinationTask::PlanEnd {
                                result: Ok(()),
                                plan_id,
                            }
                        }));
                    }
                }
                CoordinationTask::PlanEnd { plan_id, result } => {
                    if let Err(err) = result {
                        let mut response = self.response.lock().await;
                        let plan = &self.plans[plan_id];
                        let mut op_path = plan
                            .path
                            .into_iter()
                            .map(|segment| segment.operation_field_id)
                            .collect::<Vec<_>>();
                        op_path.push(
                            // Taking first field in the output. The plan path is the root position
                            // of the object we're merging fields into.
                            plan.selection_set
                                .iter()
                                .next()
                                .map(|selection| selection.operation_field_id)
                                .unwrap(),
                        );
                        let locations = op_path
                            .last()
                            .map(|id| vec![self.operation_fields[*id].pos])
                            .unwrap_or_default();
                        let path = op_path
                            .into_iter()
                            .map(|id| response.strings[self.operation_fields[id].name].to_string())
                            .collect();
                        response.add_error(path, err.to_string(), locations);
                    }
                    // start the execution of any children for which all parents have finised.
                    for executable_plan_id in self.plans.finished(plan_id) {
                        futures.push(Box::pin(async move { CoordinationTask::PlanStart(executable_plan_id) }));
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
        serde_json::to_value(
            &self
                .response
                .lock()
                .await
                .as_serializable(&self.final_read_selection_set),
        )
        .unwrap()
    }
}

enum CoordinationTask {
    PlanStart(PlanId),
    PlanEnd {
        plan_id: PlanId,
        result: Result<(), ExecutorError>,
    },
}
