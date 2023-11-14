use engine::ServerResult;
use engine_parser::types::OperationDefinition;
use schema::Schema;

use crate::{executor::ExecutorCoordinator, plan::PlannedOperation, request::Operation};

pub struct Engine {
    pub(crate) schema: Schema,
}

impl Engine {
    pub fn new(schema: Schema) -> Self {
        Self { schema }
    }

    pub async fn execute(&self, operation_definition: OperationDefinition) -> ServerResult<serde_json::Value> {
        let operation = Operation::build(&self.schema, operation_definition);
        let planned_operation = PlannedOperation::build(self, operation);
        let response = ExecutorCoordinator::new(self, planned_operation).execute().await;
        Ok(response)
    }
}
