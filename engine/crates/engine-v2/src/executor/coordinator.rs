use std::sync::Arc;

use engine_parser::types::OperationType;
use futures_locks::Mutex;
use futures_util::StreamExt;

use super::{Executor, ExecutorRequest, ResponseGraphProxy};
use crate::{
    plan::{ExecutionPlanGraph, PlanId, RequestPlan},
    response_graph::{NodeSelectionSet, ResponseGraph},
    Engine,
};

pub struct ExecutorCoordinator<'a> {
    engine: &'a Engine,
    operation_type: OperationType,
    operation_selection_set: NodeSelectionSet,
    execution_plan_graph: ExecutionPlanGraph,
    response_graph: Arc<Mutex<ResponseGraph>>,
}

impl<'a> ExecutorCoordinator<'a> {
    pub fn new(engine: &'a Engine, request_plan: RequestPlan) -> Self {
        Self {
            engine,
            operation_type: request_plan.operation_type,
            operation_selection_set: request_plan.operation_selection_set,
            response_graph: Arc::new(Mutex::new(ResponseGraph::new(request_plan.response_graph_edges))),
            execution_plan_graph: request_plan.execution_plan_graph,
        }
    }

    pub async fn execute(&mut self) -> serde_json::Value {
        let (sender, mut receiver) = futures_channel::mpsc::unbounded::<CoordinationTask>();
        for plan_id in self.execution_plan_graph.executable_plan_ids() {
            sender.unbounded_send(CoordinationTask::PlanStart(plan_id)).unwrap();
        }
        while let Some(task) = receiver.next().await {
            match task {
                CoordinationTask::PlanStart(plan_id) => {
                    let plan = &self.execution_plan_graph[plan_id];
                    if let Some(input) =
                        self.response_graph
                            .lock()
                            .await
                            .input(&self.engine.schema, &plan.path, &plan.input)
                    {
                        let resolver = &self.engine.schema[plan.resolver_id];
                        let execution_plan = Executor::build(
                            self.engine,
                            resolver,
                            ExecutorRequest {
                                operation_type: self.operation_type,
                                input,
                                output: &plan.output,
                            },
                        );
                        let sender = sender.clone();
                        let proxy = ResponseGraphProxy {
                            graph: self.response_graph.clone(),
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
                    for executable_plan_id in self.execution_plan_graph.finished(plan_id) {
                        sender
                            .unbounded_send(CoordinationTask::PlanStart(executable_plan_id))
                            .unwrap();
                    }
                    if self.execution_plan_graph.is_finished() {
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
