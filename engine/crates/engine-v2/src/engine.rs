use std::sync::Arc;

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

    pub async fn execute(&self, request: engine::Request) -> Response {
        match self.prepare(&request).await {
            Ok(operation) => match Variables::from_request(&operation, self.schema.as_ref(), request.variables) {
                Ok(variables) => {
                    let mut executor = ExecutorCoordinator::new(self, &operation, &variables);
                    executor.execute().await;
                    executor.into_response()
                }
                Err(err) => Response::from_errors(err, ExecutionMetadata::build(&operation)),
            },
            Err(err) => Response::from_error(err, ExecutionMetadata::default()),
        }
    }

    async fn prepare(&self, request: &engine::Request) -> Result<Operation, GraphqlError> {
        let unbound_operation = parse_operation(request)?;
        let operation = Operation::bind(&self.schema, unbound_operation)?;
        Ok(operation)
    }
}
