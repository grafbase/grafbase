use engine::ServerResult;

use crate::{executor::ExecutorCoordinator, plan::RequestPlan, request::OperationBinder};

pub struct Engine {
    pub(crate) schema: schema::Schema,
}

impl Engine {
    pub fn new(schema: schema::Schema) -> Self {
        Self { schema }
    }

    pub async fn execute(&self, request: engine_parser::types::OperationDefinition) -> ServerResult<serde_json::Value> {
        let operation = OperationBinder::new(&self.schema).bind(request)?;
        let request_plan = RequestPlan::builder(self).build(operation); // could be cached
        Ok(ExecutorCoordinator::new(self, request_plan).execute().await)
    }
}
