use std::{sync::Arc, time::Duration};

use async_runtime::stream::StreamExt as _;
use engine::RequestHeaders;
use engine_parser::types::OperationType;
use futures::channel::mpsc;
use futures_util::{SinkExt, Stream};
use schema::Schema;

use crate::{
    execution::{ExecutorCoordinator, PreparedExecution, PreparedRequest, Variables},
    plan::OperationPlan,
    request::{parse_operation, Operation},
    response::{ExecutionMetadata, GraphqlError, Response},
};

pub struct Engine {
    // We use an Arc for the schema to have a self-contained response which may still
    // needs access to the schema strings
    pub(crate) schema: Arc<Schema>,
    pub(crate) env: EngineEnv,
    #[cfg(feature = "plan_cache")]
    plan_cache: moka::sync::Cache<String, Arc<OperationPlan>>,
}

pub struct EngineEnv {
    pub fetcher: runtime::fetch::Fetcher,
}

impl Engine {
    pub fn new(schema: Schema, env: EngineEnv) -> Self {
        Self {
            schema: Arc::new(schema),
            env,
            #[cfg(feature = "plan_cache")]
            plan_cache: moka::sync::Cache::builder()
                .max_capacity(64)
                // A cached entry will be expired after the specified duration past from get or insert
                .time_to_idle(Duration::from_secs(5 * 60))
                .build(),
        }
    }

    pub fn execute(self: &Arc<Self>, mut request: engine::Request, headers: RequestHeaders) -> PreparedExecution {
        let operation = match self.prepare_operation(&request) {
            Ok(operation) => operation,
            Err(error) => {
                return PreparedExecution::bad_request(Response::from_error(error, ExecutionMetadata::default()))
            }
        };
        let variables = match Variables::from_request(&operation, self.schema.as_ref(), &mut request.variables) {
            Ok(variables) => variables,
            Err(errors) => {
                return PreparedExecution::bad_request(Response::from_errors(
                    errors,
                    ExecutionMetadata::build(&operation),
                ))
            }
        };

        if matches!(operation.ty, OperationType::Subscription) {
            return PreparedExecution::bad_request(Response::from_error(
                GraphqlError::new("Subscriptions are only suported on streaming transports.  Try making a request with SSE or WebSockets"),
                ExecutionMetadata::build(&operation),
            ));
        }

        PreparedExecution::PreparedRequest(PreparedRequest {
            query: request.query,
            operation,
            variables,
            headers,
            engine: Arc::clone(self),
        })
    }

    pub fn execute_stream(
        self: &Arc<Self>,
        request: engine::Request,
        headers: RequestHeaders,
    ) -> impl Stream<Item = Response> {
        let (mut sender, receiver) = mpsc::channel(2);
        let engine = Arc::clone(self);

        receiver.join(async move {
            let coordinator = match engine.prepare_coordinator(request, headers) {
                Ok(coordinator) => coordinator,
                Err(response) => {
                    sender.send(response).await.ok();
                    return;
                }
            };

            if matches!(
                coordinator.operation_type(),
                OperationType::Query | OperationType::Mutation
            ) {
                sender.send(coordinator.execute().await).await.ok();
                return;
            }

            coordinator.execute_subscription(sender).await
        })
    }

    fn prepare_coordinator(
        &self,
        mut request: engine::Request,
        headers: RequestHeaders,
    ) -> Result<ExecutorCoordinator<'_>, Response> {
        let operation = match self.prepare_operation(&request) {
            Ok(operation) => operation,
            Err(error) => return Err(Response::from_error(error, ExecutionMetadata::default())),
        };
        let variables = match Variables::from_request(&operation, self.schema.as_ref(), &mut request.variables) {
            Ok(variables) => variables,
            Err(errors) => return Err(Response::from_errors(errors, ExecutionMetadata::build(&operation))),
        };

        Ok(ExecutorCoordinator::new(self, operation, variables, headers))
    }

    fn prepare_operation(&self, request: &engine::Request) -> Result<Arc<OperationPlan>, GraphqlError> {
        #[cfg(feature = "plan_cache")]
        {
            if let Some(cached) = self.plan_cache.get(&request.query) {
                return Ok(cached);
            }
        }
        let parsed_operation = parse_operation(request)?;
        let bound_operation = Operation::build(&self.schema, parsed_operation, !request.disable_operation_limits)?;
        let prepared = Arc::new(OperationPlan::prepare(&self.schema, bound_operation)?);
        #[cfg(feature = "plan_cache")]
        {
            self.plan_cache.insert(request.query.clone(), prepared.clone())
        }
        Ok(prepared)
    }
}
