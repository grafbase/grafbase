use std::sync::Arc;

use futures::channel::mpsc;
use futures_util::{SinkExt, Stream};
use tracing::Instrument;

use async_runtime::stream::StreamExt as _;
use engine::RequestHeaders;
use engine_parser::types::OperationType;
use grafbase_tracing::span::gql::GqlRequestSpan;
use grafbase_tracing::span::{GqlRecorderSpanExt, GqlResponseAttributes};
use schema::Schema;

use crate::{
    execution::{ExecutionCoordinator, PreparedExecution},
    plan::OperationPlan,
    request::{bind_variables, Operation},
    response::{ExecutionMetadata, GraphqlError, Response},
};

mod trusted_documents;

const CLIENT_NAME_HEADER_NAME: &str = "x-grafbase-client-name";

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
    pub trusted_documents: runtime::trusted_documents_service::TrustedDocumentsClient,
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
        let gql_span = GqlRequestSpan::new().with_document(request.query()).into_span();

        let coordinator = match self.prepare_coordinator(request, headers).await {
            Ok(coordinator) => coordinator,
            Err(response) => {
                return {
                    gql_span.record_gql_response(GqlResponseAttributes { has_errors: true });
                    PreparedExecution::bad_request(response)
                }
            }
        };

        if matches!(coordinator.operation().ty, OperationType::Subscription) {
            gql_span.record_gql_response(GqlResponseAttributes { has_errors: true });

            return PreparedExecution::bad_request(Response::from_error(
                GraphqlError::new("Subscriptions are only suported on streaming transports.  Try making a request with SSE or WebSockets"),
                ExecutionMetadata::build(coordinator.operation())
            ));
        }

        PreparedExecution::request(coordinator, gql_span)
    }

    pub fn execute_stream(
        self: &Arc<Self>,
        request: engine::Request,
        headers: RequestHeaders,
    ) -> impl Stream<Item = Response> {
        let gql_span = GqlRequestSpan::new().with_document(request.query()).into_span();

        let (mut sender, receiver) = mpsc::channel(2);
        let engine = Arc::clone(self);

        receiver.join(
            async move {
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
            }
            .instrument(gql_span),
        )
    }

    async fn prepare_coordinator(
        self: &Arc<Self>,
        mut request: engine::Request,
        headers: RequestHeaders,
    ) -> Result<ExecutionCoordinator, Response> {
        // Injecting the query string if necessary.
        self.handle_persisted_query(&mut request, headers.find(CLIENT_NAME_HEADER_NAME))
            .await
            .map_err(|err| Response::from_error(err, ExecutionMetadata::default()))?;

        let operation_plan = match self.prepare_operation(&request).await {
            Ok(operation) => operation,
            Err(error) => return Err(Response::from_error(error, ExecutionMetadata::default())),
        };

        let input_values = bind_variables(self.schema.as_ref(), &operation_plan, &mut request.variables)
            .map_err(|errors| Response::from_errors(errors, ExecutionMetadata::build(&operation_plan)))?;

        Ok(ExecutionCoordinator::new(
            Arc::clone(self),
            operation_plan,
            input_values,
            headers,
        ))
    }

    async fn prepare_operation(&self, request: &engine::Request) -> Result<Arc<OperationPlan>, GraphqlError> {
        #[cfg(feature = "plan_cache")]
        {
            if let Some(cached) = self.plan_cache.get(&request.operation_plan_cache_key) {
                return Ok(cached);
            }
        }
        let operation = Operation::build(&self.schema, request)?;
        let prepared = Arc::new(OperationPlan::prepare(&self.schema, operation)?);
        #[cfg(feature = "plan_cache")]
        {
            self.plan_cache
                .insert(request.operation_plan_cache_key.clone(), prepared.clone())
        }
        Ok(prepared)
    }
}
