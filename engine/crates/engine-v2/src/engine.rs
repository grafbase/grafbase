use std::sync::Arc;

use async_runtime::stream::StreamExt as _;
use engine::{HttpGraphqlRequest, HttpGraphqlResponse, RequestExtensions, SchemaVersion};
use engine_parser::types::OperationType;
use engine_v2_common::{
    BatchGraphqlRequest, GraphqlRequest, OperationCacheControlCacheKey, ResponseCacheKey, StreamingFormat,
};
use futures::channel::mpsc;
use futures::StreamExt;
use futures_util::{SinkExt, Stream};
#[cfg(feature = "tracing")]
use grafbase_tracing::span::{gql::GqlRequestSpan, GqlRecorderSpanExt, GqlResponseAttributes};
use headers::HeaderMapExt;
use runtime::auth::AccessToken;
use schema::Schema;
#[cfg(feature = "tracing")]
use tracing::Instrument;

use crate::plan::build_execution_metadata;
use crate::{
    execution::ExecutionCoordinator,
    plan::OperationPlan,
    request::{bind_variables, Operation},
    response::{GraphqlError, Response},
};

mod trusted_documents;

const CLIENT_NAME_HEADER_NAME: &str = "x-grafbase-client-name";

pub struct Engine {
    // We use an Arc for the schema to have a self-contained response which may still
    // needs access to the schema strings
    pub(crate) schema: Arc<Schema>,
    pub(crate) schema_version: SchemaVersion,
    pub(crate) env: EngineEnv,
    #[cfg(feature = "plan_cache")]
    plan_cache: mini_moka::sync::Cache<engine::OperationPlanCacheKey, Arc<OperationPlan>>,
    // public for websockets
    #[cfg(feature = "auth")]
    pub(crate) auth: gateway_v2_auth::AuthService,
}

pub struct EngineEnv {
    pub fetcher: runtime::fetch::Fetcher,
    pub cache: runtime::cache::Cache,
    pub cache_opeartion_cache_control: bool,
    pub async_runtime: runtime::async_runtime::AsyncRuntime,
    pub trusted_documents: runtime::trusted_documents_service::TrustedDocumentsClient,
    #[cfg(feature = "auth")]
    pub kv: runtime::kv::KvStore,
}

impl Engine {
    pub fn new(schema: Schema, schema_version: SchemaVersion, env: EngineEnv) -> Self {
        #[cfg(feature = "auth")]
        let auth = gateway_v2_auth::AuthService::new_v2(schema.auth_config.clone().unwrap_or_default(), env.kv.clone());
        Self {
            schema: Arc::new(schema),
            schema_version,
            env,
            #[cfg(feature = "plan_cache")]
            plan_cache: mini_moka::sync::Cache::builder()
                .max_capacity(64)
                // A cached entry will be expired after the specified duration past from get or insert
                .time_to_idle(std::time::Duration::from_secs(5 * 60))
                .build(),
            #[cfg(feature = "auth")]
            auth,
        }
    }

    #[cfg(feature = "auth")]
    pub async fn execute(
        self: &Arc<Self>,
        headers: http::HeaderMap,
        // TODO: remove me once we have proper tracing...
        ray_id: &str,
        request: HttpGraphqlRequest<'_>,
    ) -> HttpGraphqlResponse {
        if let Some(access_token) = self.auth.authorize(&headers).await {
            self.execute_with_access_token(headers, access_token, ray_id, request)
                .await
        } else {
            HttpGraphqlResponse::error("Missing access token")
        }
    }

    pub async fn execute_with_access_token(
        self: &Arc<Self>,
        headers: http::HeaderMap,
        access_token: AccessToken,
        // TODO: remove me once we have proper tracing...
        ray_id: &str,
        request: HttpGraphqlRequest<'_>,
    ) -> HttpGraphqlResponse {
        let batch_request = match BatchGraphqlRequest::<'_, RequestExtensions>::from_http_request(&request) {
            Ok(r) => r,
            Err(message) => return HttpGraphqlResponse::error(&message),
        };

        let headers = Arc::new(headers);
        let access_token = Arc::new(access_token);
        let streaming_format = headers.typed_get::<StreamingFormat>();
        match batch_request {
            BatchGraphqlRequest::Single(request) => {
                if let Some(streaming_format) = streaming_format {
                    HttpGraphqlResponse::from_stream(
                        ray_id,
                        streaming_format,
                        self.execute_stream(headers, access_token, ray_id, request).await,
                    )
                    .await
                } else {
                    self.execute_single(headers, access_token, ray_id, request).await
                }
            }
            BatchGraphqlRequest::Batch(requests) => {
                if streaming_format.is_some() {
                    return HttpGraphqlResponse::error("batch requests can't use multipart or event-stream responses");
                }
                HttpGraphqlResponse::batch_response(
                    futures_util::stream::iter(requests.into_iter())
                        .then(|request| async {
                            self.execute_single(headers.clone(), access_token.clone(), ray_id, request)
                                .await
                        })
                        .collect::<Vec<_>>()
                        .await,
                )
                .await
            }
        }
    }

    async fn execute_single(
        self: &Arc<Self>,
        headers: Arc<http::HeaderMap>,
        access_token: Arc<AccessToken>,
        ray_id: &str,
        request: GraphqlRequest<'_, RequestExtensions>,
    ) -> HttpGraphqlResponse {
        #[cfg(feature = "tracing")]
        let gql_span = GqlRequestSpan::new()
            .with_document(request.query.as_ref().map(|q| q.as_ref()))
            .into_span();

        match self.prepare_coordinator(headers, access_token, ray_id, request).await {
            Ok(coordinator) => {
                if matches!(coordinator.operation().ty, OperationType::Subscription) {
                    #[cfg(feature = "tracing")]
                    gql_span.record_gql_response(GqlResponseAttributes { has_errors: true });
                    return Response::bad_request(GraphqlError::new(
                        "Subscriptions are only suported on streaming transports. Try making a request with SSE or WebSockets",
                    )).into();
                }

                coordinator.cached_execute().await
            }
            Err(response) => {
                #[cfg(feature = "tracing")]
                gql_span.record_gql_response(GqlResponseAttributes { has_errors: true });
                response.into()
            }
        }
    }

    // public for websockets
    pub(crate) async fn execute_stream(
        self: &Arc<Self>,
        headers: Arc<http::HeaderMap>,
        access_token: Arc<AccessToken>,
        ray_id: &str,
        request: GraphqlRequest<'_, RequestExtensions>,
    ) -> impl Stream<Item = Response> {
        #[cfg(feature = "tracing")]
        let gql_span = GqlRequestSpan::new()
            .with_document(request.query.as_ref().map(|q| q.as_ref()))
            .into_span();
        let (mut sender, receiver) = mpsc::channel(2);

        let result = self.prepare_coordinator(headers, access_token, ray_id, request).await;
        receiver.join({
            let future = async move {
                let coordinator = match result {
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
            #[cfg(feature = "tracing")]
            let future = future.instrument(gql_span);
            future
        })
    }

    async fn prepare_coordinator<'a>(
        self: &Arc<Self>,
        headers: Arc<http::HeaderMap>,
        access_token: Arc<AccessToken>,
        ray_id: &str,
        mut request: GraphqlRequest<'_, RequestExtensions>,
    ) -> Result<ExecutionCoordinator, Response> {
        // Injecting the query string if necessary.
        if let Err(err) = self
            .handle_persisted_query(&mut request, headers.as_ref(), ray_id)
            .await
        {
            return Err(Response::bad_request(err));
        }

        // TODO: remove this useless conversion
        let mut engine_request = engine::Request::build(&request, ray_id);

        let operation_plan = self
            .prepare_operation(&engine_request)
            .await
            .map_err(Response::bad_request)?;

        let response_cache_key = operation_plan
            .cache_control
            .as_ref()
            .and_then(|operation_cache_control| {
                ResponseCacheKey::build(
                    headers.as_ref(),
                    access_token.as_ref(),
                    &request,
                    operation_cache_control,
                )
            });

        let operation_cache_control_cache_key = response_cache_key
            .as_ref()
            .map(|_| OperationCacheControlCacheKey::build(&self.schema_version, &request));

        let input_values = bind_variables(self.schema.as_ref(), &operation_plan, &mut engine_request.variables)
            .map_err(|errors| Response::from_errors(errors, build_execution_metadata(&self.schema, &operation_plan)))?;

        Ok(ExecutionCoordinator::new(
            Arc::clone(self),
            headers,
            access_token,
            operation_plan,
            input_values,
            response_cache_key,
            operation_cache_control_cache_key,
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
