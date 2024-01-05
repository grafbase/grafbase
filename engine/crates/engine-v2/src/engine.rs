use std::sync::Arc;

use engine::RequestHeaders;
use schema::Schema;

use crate::{
    execution::{ExecutorCoordinator, Variables},
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

    fn prepare(&self, request: &engine::Request) -> Result<Operation, GraphqlError> {
        let unbound_operation = parse_operation(request)?;
        let operation = Operation::build(&self.schema, unbound_operation)?;
        Ok(operation)
    }
}
