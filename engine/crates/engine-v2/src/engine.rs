use engine::{ServerError, ServerResult};
use engine_parser::types::OperationDefinition;
use schema::Schema;

use crate::{executor::ExecutorCoordinator, plan::PlannedOperation};

pub struct Engine {
    pub(crate) schema: Schema,
}

impl Engine {
    pub fn new(schema: Schema) -> Self {
        Self { schema }
    }

    pub async fn execute_request(&self, request: engine::Request) -> ServerResult<serde_json::Value> {
        let document = engine_parser::parse_query(request.query)?;

        let mut operations = document.operations.iter();

        let operation = match request.operation_name {
            None => operations
                .next()
                .ok_or_else(|| ServerError::new("document contains no operations", None))?,
            Some(expected_name) => operations
                .find(|(name, _)| name.is_some() && *name.unwrap() == expected_name)
                .ok_or_else(|| ServerError::new(format!("could not find an operation named {expected_name}"), None))?,
        }
        .1
        .clone()
        .node;

        self.execute(operation).await
    }

    pub async fn execute(&self, operation_definition: OperationDefinition) -> ServerResult<serde_json::Value> {
        let planned_operation = PlannedOperation::build(self, operation_definition);
        let response = ExecutorCoordinator::new(self, planned_operation).execute().await;
        Ok(response)
    }
}
