use std::sync::Arc;

use engine::RequestHeaders;
use runtime::cache::{CacheMetadata, Cacheable, CachedExecutionResponse, GlobalCacheConfig, RequestCacheConfig};
use schema::Schema;

use crate::{
    execution::{ExecutorCoordinator, Variables},
    request::{parse_operation, Operation},
    response::{CacheableResponse, ExecutionMetadata, GraphqlError, Response},
};

pub struct Engine {
    // We use an Arc for the schema to have a self-contained response which may still
    // needs access to the schema strings
    pub(crate) schema: Arc<Schema>,
    pub(crate) global_cache_config: GlobalCacheConfig,
    pub(crate) runtime: EngineRuntime,
}

pub struct EngineRuntime {
    pub fetcher: runtime::fetch::Fetcher,
    pub cache: runtime::cache::Cache,
}

impl Engine {
    pub fn new(schema: Schema, runtime: EngineRuntime) -> Self {
        Self {
            schema: Arc::new(schema),
            runtime,
            global_cache_config: todo!(),
        }
    }

    pub async fn cached_json_execute(
        &self,
        request: engine::Request,
        headers: RequestHeaders,
    ) -> CachedExecutionResponse<CacheableResponse> {
        let request_cache_config = RequestCacheConfig::default();
        runtime::cache::cached_execution(
            self.runtime.cache.clone(),
            &self.global_cache_config,
            &request_cache_config,
            String::new(),
            todo!(),
            async move {
                let (operation, response) = self.execute(request, headers).await;
                let response: CacheableResponse = response.into();
                let metadata: CacheMetadata = todo!();
                Result::<_, String>::Ok((response, metadata))
            },
        )
        .await
        .expect("never errors")
    }

    async fn execute(&self, request: engine::Request, headers: RequestHeaders) -> (Option<Operation>, Response) {
        let operation = match self.prepare(&request).await {
            Ok(operation) => operation,
            Err(error) => return (None, Response::from_error(error, ExecutionMetadata::default())),
        };
        let variables = match Variables::from_request(&operation, self.schema.as_ref(), request.variables) {
            Ok(variables) => variables,
            Err(errors) => {
                return (
                    None,
                    Response::from_errors(errors, ExecutionMetadata::build(&operation)),
                )
            }
        };

        let mut executor = ExecutorCoordinator::new(self, &operation, &variables, &headers);
        executor.execute().await;
        let response = executor.into_response();
        (Some(operation), response)
    }

    async fn prepare(&self, request: &engine::Request) -> Result<Operation, GraphqlError> {
        let unbound_operation = parse_operation(request)?;
        let operation = Operation::build(&self.schema, unbound_operation)?;
        Ok(operation)
    }
}
