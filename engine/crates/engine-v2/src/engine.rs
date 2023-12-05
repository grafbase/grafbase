use std::sync::Arc;

use schema::Schema;

use crate::{
    error::EngineError,
    execution::{ExecutorCoordinator, Variables},
    request::{parse_operation, Operation},
    response::Response,
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
            Ok(operation) => match Variables::from_request(&operation, request.variables) {
                Ok(variables) => {
                    let mut executor = ExecutorCoordinator::new(self, &operation, &variables);
                    executor.execute().await;
                    executor.into_response()
                }
                Err(err) => Response::from_error(err),
            },
            Err(err) => Response::from_error(err),
        }
    }

    async fn prepare(&self, request: &engine::Request) -> Result<Operation, EngineError> {
        let unbound_operation = parse_operation(request)?;
        let operation = Operation::bind(&self.schema, unbound_operation)?;
        Ok(operation)
    }
}
