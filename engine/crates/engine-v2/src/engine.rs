use std::sync::Arc;

use async_runtime::stream::StreamExt as _;
use engine::{AutomaticPersistedQuery, ErrorCode, PersistedQueryRequestExtension, RequestHeaders};
use engine_parser::types::OperationType;
use futures::channel::mpsc;
use futures_util::{SinkExt, Stream};
use schema::Schema;

use crate::{
    execution::{ExecutionCoordinator, PreparedExecution, Variables},
    plan::OperationPlan,
    request::{parse_operation, Operation},
    response::{ExecutionMetadata, GraphqlError, Response},
};

pub struct Engine {
    // We use an Arc for the schema to have a self-contained response which may still
    // needs access to the schema strings
    pub(crate) schema: Arc<Schema>,
    pub(crate) env: EngineEnv,
    #[cfg(feature = "plan_cache")]
    plan_cache: mini_moka::sync::Cache<engine::OperationPlanCacheKey, Arc<OperationPlan>>,
}

pub struct EngineEnv {
    pub fetcher: runtime::fetch::Fetcher,
    pub cache: runtime::cache::Cache,
}

impl Engine {
    pub fn new(schema: Schema, env: EngineEnv) -> Self {
        Self {
            schema: Arc::new(schema),
            env,
            #[cfg(feature = "plan_cache")]
            plan_cache: mini_moka::sync::Cache::builder()
                .max_capacity(64)
                // A cached entry will be expired after the specified duration past from get or insert
                .time_to_idle(std::time::Duration::from_secs(5 * 60))
                .build(),
        }
    }

    pub async fn execute(self: &Arc<Self>, request: engine::Request, headers: RequestHeaders) -> PreparedExecution {
        let coordinator = match self.prepare_coordinator(request, headers).await {
            Ok(coordinator) => coordinator,
            Err(response) => return PreparedExecution::bad_request(response),
        };

        if matches!(coordinator.operation().ty, OperationType::Subscription) {
            return PreparedExecution::bad_request(Response::from_error(
                GraphqlError::new("Subscriptions are only suported on streaming transports.  Try making a request with SSE or WebSockets"),
                ExecutionMetadata::build(coordinator.operation())
            ));
        }

        PreparedExecution::request(coordinator)
    }

    pub fn execute_stream(
        self: &Arc<Self>,
        request: engine::Request,
        headers: RequestHeaders,
    ) -> impl Stream<Item = Response> {
        let (mut sender, receiver) = mpsc::channel(2);
        let engine = Arc::clone(self);

        receiver.join(async move {
            let coordinator = match engine.prepare_coordinator(request, headers).await {
                Ok(coordinator) => coordinator,
                Err(response) => {
                    sender.send(response).await.ok();
                    return;
                }
            };

            if matches!(
                coordinator.operation().ty,
                OperationType::Query | OperationType::Mutation
            ) {
                sender.send(coordinator.execute().await).await.ok();
                return;
            }

            coordinator.execute_subscription(sender).await
        })
    }

    async fn prepare_coordinator(
        self: &Arc<Self>,
        mut request: engine::Request,
        headers: RequestHeaders,
    ) -> Result<ExecutionCoordinator, Response> {
        let operation_plan = match self.prepare_operation(&mut request).await {
            Ok(operation) => operation,
            Err(error) => return Err(Response::from_error(error, ExecutionMetadata::default())),
        };
        let variables = match Variables::from_request(&operation_plan, self.schema.as_ref(), &mut request.variables) {
            Ok(variables) => variables,
            Err(errors) => return Err(Response::from_errors(errors, ExecutionMetadata::build(&operation_plan))),
        };

        Ok(ExecutionCoordinator::new(
            Arc::clone(self),
            request.operation_plan_cache_key,
            operation_plan,
            variables,
            headers,
        ))
    }

    async fn prepare_operation(&self, request: &mut engine::Request) -> Result<Arc<OperationPlan>, GraphqlError> {
        self.handle_persisted_query(request).await?;

        #[cfg(feature = "plan_cache")]
        {
            if let Some(cached) = self.plan_cache.get(&request.operation_plan_cache_key) {
                return Ok(cached);
            }
        }
        let parsed_operation = parse_operation(request)?;
        let bound_operation = Operation::build(
            &self.schema,
            parsed_operation,
            !request.operation_limits_disabled(),
            request.introspection_state(),
        )?;
        let prepared = Arc::new(OperationPlan::prepare(&self.schema, bound_operation)?);
        #[cfg(feature = "plan_cache")]
        {
            self.plan_cache
                .insert(request.operation_plan_cache_key.clone(), prepared.clone())
        }
        Ok(prepared)
    }

    async fn handle_persisted_query(&self, request: &mut engine::Request) -> Result<(), GraphqlError> {
        let Some(PersistedQueryRequestExtension { version, sha256_hash }) = &request.extensions.persisted_query else {
            return Ok(());
        };

        if *version != 1 {
            return Err(GraphqlError::new("Persisted query version not supported"));
        }

        let cache = &self.env.cache;
        let key = cache.build_key(&format!("apq/sha256_{}", hex::encode(sha256_hash)));
        if !request.query().is_empty() {
            use sha2::{Digest, Sha256};
            let digest = <Sha256 as Digest>::digest(request.query().as_bytes()).to_vec();
            if &digest != sha256_hash {
                return Err(GraphqlError::new("Invalid persisted query sha256Hash"));
            }
            cache
                .put_json(
                    &key,
                    runtime::cache::EntryState::Fresh,
                    &AutomaticPersistedQuery::V1 {
                        query: request.query().to_string(),
                    },
                    runtime::cache::CacheMetadata {
                        max_age: std::time::Duration::from_secs(24 * 60 * 60),
                        stale_while_revalidate: std::time::Duration::ZERO,
                        tags: Vec::new(),
                        should_purge_related: false,
                        should_cache: true,
                    },
                )
                .await
                .map_err(|err| {
                    log::error!(request.ray_id, "Cache error: {}", err);
                    GraphqlError::internal_server_error()
                })?;
            return Ok(());
        }

        match cache.get_json::<AutomaticPersistedQuery>(&key).await {
            Ok(entry) => {
                if let Some(AutomaticPersistedQuery::V1 { query }) = entry.into_value() {
                    request.operation_plan_cache_key.query = query;
                    Ok(())
                } else {
                    Err(GraphqlError::new("Persisted query not found")
                        .with_error_code(ErrorCode::PersistedQueryNotFound))
                }
            }
            Err(err) => {
                log::error!(request.ray_id, "Cache error: {}", err);
                Err(GraphqlError::internal_server_error())
            }
        }
    }
}
