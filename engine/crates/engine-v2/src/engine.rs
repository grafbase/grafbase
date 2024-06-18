use std::sync::Arc;
use web_time::Instant;

use async_runtime::stream::StreamExt as _;
use engine::{BatchRequest, Request};
use engine_parser::types::OperationType;
use futures::{channel::mpsc, StreamExt};
use futures_util::{SinkExt, Stream};
use gateway_core::StreamingFormat;
use gateway_v2_auth::AuthService;
use grafbase_tracing::{
    gql_response_status::GraphqlResponseStatus,
    grafbase_client::Client,
    metrics::{GraphqlOperationMetrics, GraphqlOperationMetricsAttributes},
    span::{gql::GqlRequestSpan, GqlRecorderSpanExt, GqlRequestAttributes},
};
use headers::HeaderMapExt;
use runtime::auth::AccessToken;
use schema::Schema;
use tracing::{Instrument, Span};

use crate::{
    execution::{ExecutionContext, ExecutionCoordinator},
    http_response::{HttpGraphqlResponse, OperationMetadata},
    operation::{Operation, Variables},
    plan::OperationPlan,
    response::{GraphqlError, Response},
    websocket,
};

mod trusted_documents;

pub struct Engine {
    // We use an Arc for the schema to have a self-contained response which may still
    // needs access to the schema strings
    pub(crate) schema: Arc<Schema>,
    pub(crate) env: EngineEnv,
    operation_metrics: GraphqlOperationMetrics,
    auth: AuthService,
}

pub struct EngineEnv {
    pub fetcher: runtime::fetch::Fetcher,
    pub cache: runtime::cache::Cache,
    pub trusted_documents: runtime::trusted_documents_client::Client,
    pub kv: runtime::kv::KvStore,
    pub meter: grafbase_tracing::otel::opentelemetry::metrics::Meter,
}

impl Engine {
    pub fn new(schema: Arc<Schema>, env: EngineEnv) -> Self {
        let auth = gateway_v2_auth::AuthService::new_v2(
            schema.settings.auth_config.clone().unwrap_or_default(),
            env.kv.clone(),
        );
        Self {
            schema,
            auth,
            operation_metrics: GraphqlOperationMetrics::build(&env.meter),
            env,
        }
    }

    pub async fn execute(
        self: &Arc<Self>,
        headers: http::HeaderMap,
        batch_request: BatchRequest,
    ) -> HttpGraphqlResponse {
        if let Some(access_token) = self.auth.authorize(&headers).await {
            self.execute_with_access_token(RequestMetadata::new(headers, access_token), batch_request)
                .await
        } else if let Some(streaming_format) = headers.typed_get::<StreamingFormat>() {
            HttpGraphqlResponse::stream_error(streaming_format, "Unauthorized")
        } else {
            HttpGraphqlResponse::error("Unauthorized")
        }
    }

    pub async fn create_session(self: &Arc<Self>, headers: http::HeaderMap) -> Option<Session> {
        self.auth.authorize(&headers).await.map(|access_token| Session {
            engine: Arc::clone(self),
            metadata: Arc::new(RequestMetadata::new(headers, access_token)),
        })
    }

    async fn execute_with_access_token(
        self: &Arc<Self>,
        request_metadata: RequestMetadata,
        batch_request: BatchRequest,
    ) -> HttpGraphqlResponse {
        let streaming_format = request_metadata.headers.typed_get::<StreamingFormat>();
        match batch_request {
            BatchRequest::Single(request) => {
                if let Some(streaming_format) = streaming_format {
                    HttpGraphqlResponse::from_stream(
                        streaming_format,
                        self.execute_stream(Arc::new(request_metadata), request),
                    )
                } else {
                    self.execute_single(&request_metadata, request).await
                }
            }
            BatchRequest::Batch(requests) => {
                if streaming_format.is_some() {
                    return HttpGraphqlResponse::error("batch requests can't use multipart or event-stream responses");
                }
                HttpGraphqlResponse::batch_response(
                    futures_util::stream::iter(requests.into_iter())
                        .then(|request| self.execute_single(&request_metadata, request))
                        .collect::<Vec<_>>()
                        .await,
                )
            }
        }
    }

    async fn execute_single(&self, request_metadata: &RequestMetadata, request: Request) -> HttpGraphqlResponse {
        let span = GqlRequestSpan::new().into_span();
        let start = Instant::now();
        let ctx = ExecutionContext {
            engine: self,
            request_metadata,
        };
        let (metrics_attributes, response) = ctx.execute_single(span.clone(), request).instrument(span).await;

        let mut metadata = OperationMetadata {
            operation_name: None,
            operation_type: None,
            has_errors: response.has_errors(),
        };
        if let Some(metrics_attributes) = metrics_attributes {
            metadata.operation_name.clone_from(&metrics_attributes.name);
            metadata.operation_type = Some(metrics_attributes.ty);
            self.operation_metrics.record(metrics_attributes, start.elapsed());
        }

        HttpGraphqlResponse::from(response).with_metadata(metadata)
    }

    fn execute_stream(
        self: &Arc<Self>,
        request_metadata: Arc<RequestMetadata>,
        request: Request,
    ) -> impl Stream<Item = Response> {
        let engine = Arc::clone(self);
        let span = GqlRequestSpan::new().into_span();
        let (sender, receiver) = mpsc::channel(2);

        receiver.join(async move {
            let start = Instant::now();
            let ctx = ExecutionContext {
                engine: &engine,
                request_metadata: &request_metadata,
            };
            let metrics_attributes = ctx.execute_stream(span.clone(), request, sender).instrument(span).await;

            if let Some(metrics_attributes) = metrics_attributes {
                engine.operation_metrics.record(metrics_attributes, start.elapsed());
            }
        })
    }
}

impl<'ctx> ExecutionContext<'ctx> {
    async fn execute_single(
        self,
        span: Span,
        mut request: Request,
    ) -> (Option<GraphqlOperationMetricsAttributes>, Response) {
        let operation = match self.prepare_operation(&mut request).await {
            Ok(operation) => operation,
            Err(err) => {
                return (None, Response::from_error(err));
            }
        };

        // Same behavior as in our workers for now
        // If the query didn't parse, or didn't have the named/unnamed operation (or any operations),
        // we skip it from the analytics.
        let metrics_attributes = operation_normalizer::normalize(request.query(), request.operation_name())
            .ok()
            .map(|normalized_query| GraphqlOperationMetricsAttributes {
                normalized_query_hash: blake3::hash(normalized_query.as_bytes()).into(),
                name: operation.name.clone(),
                ty: operation.ty.as_str(),
                normalized_query,
                status: GraphqlResponseStatus::Success,
                cache_status: None,
                client: self.request_metadata.client.clone(),
            });
        span.record_gql_request(GqlRequestAttributes {
            operation_type: operation.ty.as_str(),
            operation_name: operation.name.clone(),
        });

        let response = if matches!(operation.ty, OperationType::Subscription) {
            Response::from_error(GraphqlError::new(
                "Subscriptions are only suported on streaming transports. Try making a request with SSE or WebSockets",
            ))
        } else {
            match self.prepare_coordinator(operation, request.variables) {
                Ok(coordinator) => coordinator.execute().await,
                Err(errors) => Response::from_errors(errors),
            }
        };

        (metrics_attributes, response)
    }

    async fn execute_stream(
        self,
        span: Span,
        mut request: Request,
        mut sender: mpsc::Sender<Response>,
    ) -> Option<GraphqlOperationMetricsAttributes> {
        let operation = match self.prepare_operation(&mut request).await {
            Ok(operation) => operation,
            Err(err) => {
                sender.send(Response::from_error(err)).await.ok();
                return None;
            }
        };
        // Same behavior as in our workers for now
        // If the query didn't parse, or didn't have the named/unnamed operation (or any operations),
        // we skip it from the analytics.
        let metrics_attributes = operation_normalizer::normalize(request.query(), request.operation_name())
            .ok()
            .map(|normalized_query| GraphqlOperationMetricsAttributes {
                normalized_query_hash: blake3::hash(normalized_query.as_bytes()).into(),
                name: operation.name.clone(),
                ty: operation.ty.as_str(),
                normalized_query,
                status: GraphqlResponseStatus::Success,
                cache_status: None,
                client: self.request_metadata.client.clone(),
            });
        span.record_gql_request(GqlRequestAttributes {
            operation_type: operation.ty.as_str(),
            operation_name: operation.name.clone(),
        });

        let coordinator = match self.prepare_coordinator(operation, request.variables) {
            Ok(coordinator) => coordinator,
            Err(errors) => {
                sender.send(Response::from_errors(errors)).await.ok();
                return metrics_attributes;
            }
        };

        if matches!(
            coordinator.operation().ty,
            OperationType::Query | OperationType::Mutation
        ) {
            sender.send(coordinator.execute().await).await.ok();
            return metrics_attributes;
        }

        struct Sender {
            sender: mpsc::Sender<Response>,
        }

        impl crate::execution::ResponseSender for Sender {
            type Error = mpsc::SendError;
            async fn send(&mut self, response: Response) -> Result<(), Self::Error> {
                self.sender.send(response).await
            }
        }

        coordinator.execute_subscription(Sender { sender }).await;
        metrics_attributes
    }

    fn prepare_coordinator(
        self,
        operation: Operation,
        variables: engine::Variables,
    ) -> Result<ExecutionCoordinator<'ctx>, Vec<GraphqlError>> {
        let variables = Variables::build(self.schema.as_ref(), &operation, variables)
            .map_err(|errors| errors.into_iter().map(Into::into).collect::<Vec<_>>())?;

        let operation_plan =
            Arc::new(OperationPlan::prepare(&self.schema, &variables, operation).map_err(|err| vec![err.into()])?);

        Ok(ExecutionCoordinator::new(self, operation_plan, variables))
    }

    async fn prepare_operation(self, request: &mut engine::Request) -> Result<Operation, GraphqlError> {
        self.handle_persisted_query(request).await?;
        let operation = Operation::build(self, request)?;
        Ok(operation)
    }
}

#[derive(Clone)]
pub struct Session {
    engine: Arc<Engine>,
    metadata: Arc<RequestMetadata>,
}

pub(crate) struct RequestMetadata {
    pub headers: http::HeaderMap,
    pub client: Option<Client>,
    pub access_token: AccessToken,
}

impl RequestMetadata {
    fn new(headers: http::HeaderMap, access_token: AccessToken) -> Self {
        let client = Client::extract_from(&headers);
        Self {
            headers,
            client,
            access_token,
        }
    }
}

impl Session {
    pub fn execute_websocket(&self, id: String, request: Request) -> impl Stream<Item = websocket::Message> {
        self.engine
            .execute_stream(self.metadata.clone(), request)
            .map(move |response| match response {
                Response::BadRequest(_) => websocket::Message::Error {
                    id: id.clone(),
                    payload: websocket::Payload(response),
                },
                response => websocket::Message::Next {
                    id: id.clone(),
                    payload: websocket::Payload(response),
                },
            })
    }
}
