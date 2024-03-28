use std::sync::Arc;

use futures::channel::mpsc;
use futures_util::{SinkExt, Stream};
use runtime::auth::AccessToken;
use tracing::Instrument;

use async_runtime::stream::StreamExt as _;
use engine::RequestHeaders;
use engine_parser::types::OperationType;
use grafbase_tracing::span::{gql::GqlRequestSpan, GqlRecorderSpanExt, GqlResponseAttributes};
use schema::Schema;

use crate::{
    execution::{ExecutionContext, ExecutionCoordinator, PreparedExecution},
    operation::{Operation, Variables},
    plan::OperationPlan,
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
    pub trusted_documents: runtime::trusted_documents_client::Client,
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

    pub async fn execute(
        self: &Arc<Self>,
        request: engine::Request,
        access_token: AccessToken,
        headers: RequestHeaders,
    ) -> PreparedExecution {
        let gql_span = GqlRequestSpan::new().with_document(request.query()).into_span();

        let coordinator = match self.prepare_coordinator(request, access_token, headers).await {
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
        access_token: AccessToken,
        headers: RequestHeaders,
    ) -> impl Stream<Item = Response> {
        let gql_span = GqlRequestSpan::new().with_document(request.query()).into_span();

        let (mut sender, receiver) = mpsc::channel(2);
        let engine = Arc::clone(self);

        receiver.join({
            let future = async move {
                let coordinator = match engine.prepare_coordinator(request, access_token, headers).await {
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
            };

            future.instrument(gql_span)
        })
    }

    async fn prepare_coordinator(
        self: &Arc<Self>,
        mut request: engine::Request,
        access_token: AccessToken,
        headers: RequestHeaders,
    ) -> Result<ExecutionCoordinator, Response> {
        self.handle_persisted_query(&mut request, headers.find(CLIENT_NAME_HEADER_NAME), &headers)
            .await
            .map_err(|err| Response::from_error(err, ExecutionMetadata::default()))?;

        let (operation_plan, variables) = self
            .prepare_operation(
                ExecutionContext {
                    engine: self.as_ref(),
                    access_token: &access_token,
                    headers: &headers,
                },
                request,
            )
            .await?;

        Ok(ExecutionCoordinator::new(
            Arc::clone(self),
            operation_plan,
            variables,
            access_token,
            headers,
        ))
    }

    async fn prepare_operation(
        &self,
        ctx: ExecutionContext<'_>,
        request: engine::Request,
    ) -> Result<(Arc<OperationPlan>, Variables), Response> {
        #[cfg(feature = "plan_cache")]
        {
            if let Some(operation_plan) = self.plan_cache.get(&request.operation_plan_cache_key) {
                crate::operation::validate_cached_operation(ctx, &operation_plan)
                    .map_err(|err| Response::from_error(err, ExecutionMetadata::build(&operation_plan)))?;
                let variables = Variables::build(self.schema.as_ref(), &operation_plan, request.variables)
                    .map_err(|errors| Response::from_errors(errors, ExecutionMetadata::build(&operation_plan)))?;
                return Ok((operation_plan, variables));
            }
        }
        let operation =
            Operation::build(ctx, &request).map_err(|err| Response::from_error(err, ExecutionMetadata::default()))?;

        let variables = Variables::build(self.schema.as_ref(), &operation, request.variables)
            .map_err(|errors| Response::from_errors(errors, ExecutionMetadata::build(&operation)))?;

        let operation_plan = Arc::new(
            OperationPlan::prepare(&self.schema, &variables, operation)
                .map_err(|err| Response::from_error(err, ExecutionMetadata::default()))?,
        );
        #[cfg(feature = "plan_cache")]
        {
            self.plan_cache
                .insert(request.operation_plan_cache_key.clone(), operation_plan.clone())
        }
        Ok((operation_plan, variables))
    }
}
