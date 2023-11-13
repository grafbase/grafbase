use std::sync::Arc;

use engine::ServerResult;
use schema::Schema;

use crate::{executor::ExecutorCoordinator, plan::RequestPlan, request::OperationBinder};

pub struct Engine {
    pub(crate) schema: Arc<Schema>,
}

impl Engine {
    pub fn new(schema: Schema) -> Self {
        Self {
            schema: Arc::new(schema),
        }
    }

    pub async fn execute(&self, request: engine_parser::types::OperationDefinition) -> ServerResult<serde_json::Value> {
        let operation = OperationBinder::new(&self.schema).bind(request)?;
        let request_plan = RequestPlan::builder(self).build(operation); // could be cached
        Ok(ExecutorCoordinator::new(self, request_plan).execute().await)
    }
}
