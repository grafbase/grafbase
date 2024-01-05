use std::sync::Arc;

use engine::RequestHeaders;
use engine_parser::types::OperationType;
use futures::channel::mpsc;
use futures_util::{
    future::{BoxFuture, Fuse},
    FutureExt, Stream, StreamExt,
};
use schema::Schema;

use crate::{
    execution::{ExecutorCoordinator, ResponseReceiver, Variables},
    request::{parse_operation, Operation},
    response::{ExecutionMetadata, GraphqlError, Response},
};

pub struct Engine {
    // We use an Arc for the schema to have a self-contained response which may still
    // needs access to the schema strings
    pub(crate) schema: Arc<Schema>,
    pub(crate) runtime: EngineRuntime,
}

pub struct EngineRuntime {
    pub fetcher: runtime::fetch::Fetcher,
}

impl Engine {
    pub fn new(schema: Schema, runtime: EngineRuntime) -> Self {
        Self {
            schema: Arc::new(schema),
            runtime,
        }
    }

    pub async fn execute(&self, request: engine::Request, headers: RequestHeaders) -> Response {
        let coordinator = match self.prepare(request, headers) {
            Ok(ok) => ok,
            Err(response) => return response,
        };

        if matches!(coordinator.operation_type(), OperationType::Subscription) {
            return Response::from_error(
                GraphqlError::new("Subscriptions are only suported on streaming transports.  Try making a request with SSE or WebSockets"),
                ExecutionMetadata::default(),
            );
        }

        coordinator.execute().await
    }

    pub fn execute_stream(
        &self,
        request: engine::Request,
        headers: RequestHeaders,
    ) -> impl Stream<Item = Response> + '_ {
        let initial_state = StreamState::Starting(request, headers, self);
        futures_util::stream::unfold(initial_state, stream_handler)
    }

    fn prepare(&self, request: engine::Request, headers: RequestHeaders) -> Result<ExecutorCoordinator<'_>, Response> {
        let operation = match self.prepare_operation(&request) {
            Ok(operation) => operation,
            Err(error) => return Err(Response::from_error(error, ExecutionMetadata::default())),
        };
        let variables = match Variables::from_request(&operation, self.schema.as_ref(), request.variables) {
            Ok(variables) => variables,
            Err(errors) => return Err(Response::from_errors(errors, ExecutionMetadata::build(&operation))),
        };

        Ok(ExecutorCoordinator::new(self, operation, variables, headers))
    }

    fn prepare_operation(&self, request: &engine::Request) -> Result<Operation, GraphqlError> {
        let unbound_operation = parse_operation(request)?;
        let operation = Operation::build(&self.schema, unbound_operation)?;
        Ok(operation)
    }
}

enum StreamState<'a> {
    Starting(engine::Request, RequestHeaders, &'a Engine),
    Running(ResponseReceiver, Fuse<BoxFuture<'a, ()>>),
    Draining(ResponseReceiver),
    Finished,
}

async fn stream_handler(mut state: StreamState<'_>) -> Option<(Response, StreamState<'_>)> {
    loop {
        match state {
            StreamState::Starting(request, headers, engine) => {
                let coordinator = match engine.prepare(request, headers) {
                    Ok(coordinator) => coordinator,
                    Err(response) => return Some((response, StreamState::Finished)),
                };

                if matches!(
                    coordinator.operation_type(),
                    OperationType::Query | OperationType::Mutation
                ) {
                    let response = coordinator.execute().await;
                    return Some((response, StreamState::Finished));
                }

                let (sender, receiver) = mpsc::channel(2);

                let subscription_future: BoxFuture<'_, ()> = Box::pin(coordinator.execute_subscription(sender));

                // Pass off to the Running handler
                state = StreamState::Running(receiver, subscription_future.fuse());
            }
            StreamState::Running(mut receiver, mut subscription_future) => {
                futures::select! {
                    _ = subscription_future => {
                        // Pass off to the Draining handler to make sure we deliver any pending
                        // messages before we finish
                        state = StreamState::Draining(receiver)
                    },
                    next = receiver.next() => {
                        return Some((next?, StreamState::Running(receiver, subscription_future)));
                    }
                }
            }
            StreamState::Draining(mut receiver) => {
                return Some((receiver.next().await?, StreamState::Draining(receiver)));
            }
            StreamState::Finished => return None,
        }
    }
}
