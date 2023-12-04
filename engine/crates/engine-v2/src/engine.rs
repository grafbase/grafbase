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

    // Cloudflare Workers only.
    // Domain requests against which should be routed through a service binding.
    #[cfg(feature = "cf-workers")]
    pub(crate) self_domain_configuration: Option<(String, send_wrapper::SendWrapper<worker::Fetcher>)>,
}

impl Engine {
    pub fn new(schema: Schema) -> Self {
        #[cfg(feature = "cf-workers")]
        return Self {
            schema: Arc::new(schema),
            self_domain_configuration: None,
        };

        #[cfg(not(feature = "cf-workers"))]
        return Self {
            schema: Arc::new(schema),
        };
    }

    #[cfg(feature = "cf-workers")]
    pub fn new_with_self_domain_routing(schema: Schema, self_domain: String, service: worker::Fetcher) -> Self {
        Self {
            schema: Arc::new(schema),
            self_domain_configuration: Some((self_domain, send_wrapper::SendWrapper::new(service))),
        }
    }

    pub async fn execute(&self, request: engine::Request) -> Response {
        match self.prepare(&request).await {
            Ok(operation) => match Variables::from_request(&operation, request.variables) {
                Ok(variables) => {
                    let mut executor = ExecutorCoordinator::new(self, &operation, variables);
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
