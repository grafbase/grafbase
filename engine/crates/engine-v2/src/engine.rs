use schema::Schema;

use crate::{
    error::EngineError,
    executor::ExecutorCoordinator,
    plan::OperationPlan,
    request::{parse_operation, Operation},
    response::Response,
};

pub struct Engine {
    pub(crate) schema: Schema,
}

impl Engine {
    pub fn new(schema: Schema) -> Self {
        Self { schema }
    }

    pub async fn execute(&self, request: engine::Request) -> Response<'_> {
        match self.prepare(request).await {
            Ok(plan) => {
                let mut executor = ExecutorCoordinator::new(self, &plan);
                executor.execute().await;
                executor.get_response()
            }
            Err(err) => Response::from_error(err),
        }
    }

    async fn prepare(&self, request: engine::Request) -> Result<OperationPlan, EngineError> {
        let unbound_operation = parse_operation(&request)?;
        let operation = Operation::bind(&self.schema, unbound_operation)?;
        let plan = OperationPlan::prepare(self, operation)?;
        Ok(plan)
    }
}
