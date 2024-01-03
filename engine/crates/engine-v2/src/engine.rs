use std::sync::Arc;

use engine::RequestHeaders;
use engine_parser::types::OperationType;
use futures::channel::mpsc;
use futures_util::{future::BoxFuture, Stream};
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
        let operation = match self.prepare(&request) {
            Ok(operation) => operation,
            Err(error) => return Response::from_error(error, ExecutionMetadata::default()),
        };
        let variables = match Variables::from_request(&operation, self.schema.as_ref(), request.variables) {
            Ok(variables) => variables,
            Err(errors) => return Response::from_errors(errors, ExecutionMetadata::build(&operation)),
        };

        let executor = ExecutorCoordinator::new(self, operation, variables, headers);
        executor.execute().await
    }

    pub fn execute_stream(
        &self,
        request: engine::Request,
        headers: RequestHeaders,
    ) -> impl Stream<Item = Response> + '_ {
        let initial_state = StreamState::Starting(request, headers);
        futures_util::stream::unfold(initial_state, move |state| async move {})
    }

    fn prepare(&self, request: &engine::Request) -> Result<Operation, GraphqlError> {
        let unbound_operation = parse_operation(request)?;
        // TODO: Can possibly move Variables::from_request here to make things nicer.
        // Or package up Operation w/ Variables at least?  Not sure.
        let operation = Operation::build(&self.schema, unbound_operation)?;
        Ok(operation)
    }
}

enum StreamState<'a> {
    Starting(&'a engine::Request, &'a RequestHeaders, &'a Engine),
    Started(ResponseReceiver, BoxFuture<'a, ()>),
    Finished,
}

impl StreamState<'_> {
    pub async fn handle(self) -> Option<(Response, Self)> {
        match self {
            StreamState::Starting(request, headers, engine) => {
                let operation = match engine.prepare(&request) {
                    Ok(operation) => operation,
                    Err(error) => {
                        return Some((
                            Response::from_error(error, ExecutionMetadata::default()),
                            StreamState::Finished,
                        ))
                    }
                };

                let variables = match Variables::from_request(&operation, engine.schema.as_ref(), request.variables) {
                    Ok(variables) => variables,
                    Err(errors) => {
                        return Some((
                            Response::from_errors(errors, ExecutionMetadata::build(&operation)),
                            StreamState::Finished,
                        ))
                    }
                };

                    if matches!(operation.ty, OperationType::Query | OperationType::Mutation) {
                        let response = ExecutorCoordinator::new(engine, operation, variables, headers)
                            .execute()
                            .await;
                        return Some((response, StreamState::Finished));
                    }

                // TODO: Could probably write some tests of running queries & mutations via execute_stream
                // now...

                let executor = ExecutorCoordinator::new(engine, &operation, &variables, &headers);
                let (sender, receiver) = mpsc::channel(2);

                // Pass off to the Started handler
                StreamState::Started(receiver, Box::pin(executor.execute_subscription(sender)))
                    .handle()
                    .await
            }
            StreamState::Started(receiver, subscription_future) => {
                todo!()
            }
            StreamState::Finished => None,
        }
    }
}
