use ::runtime::{
    auth::AccessToken,
    error::ErrorResponse,
    hooks::Hooks,
    operation_cache::{OperationCache, OperationCacheFactory},
    rate_limiting::RateLimitKey,
};
use async_runtime::stream::StreamExt as _;
use bytes::Bytes;
use engine_parser::types::OperationType;
use futures::{channel::mpsc, FutureExt, StreamExt, TryFutureExt};
use futures_util::{SinkExt, Stream};
use gateway_v2_auth::AuthService;
use grafbase_telemetry::{
    gql_response_status::GraphqlResponseStatus,
    grafbase_client::Client,
    metrics::{
        GraphqlOperationMetrics, GraphqlRequestMetricsAttributes, OperationMetricsAttributes,
        QueryPreparationAttributes,
    },
    span::{gql::GqlRequestSpan, GqlRecorderSpanExt, GRAFBASE_TARGET},
};
use retry_budget::RetryBudgets;
use schema::Schema;
use std::{borrow::Cow, future::Future, sync::Arc};
use tracing::Instrument;
use trusted_documents::OperationDocument;
use web_time::{Instant, SystemTime};

use crate::{
    execution::{ExecutableOperation, PreExecutionContext},
    graphql_over_http::{Http, ResponseFormat, StreamingResponseFormat},
    operation::{Operation, PreparedOperation, Variables},
    request::{BatchRequest, QueryParamsRequest, Request},
    response::{GraphqlError, RefusedRequestResponse, RequestErrorResponse, Response},
    websocket, Body, ErrorCode,
};
use runtime::RuntimeExt;

mod cache;
mod error_responses;
mod retry_budget;
mod runtime;
mod trusted_documents;

pub use runtime::Runtime;

pub(crate) struct SchemaVersion(Vec<u8>);

impl std::ops::Deref for SchemaVersion {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct Engine<R: Runtime> {
    // We use an Arc for the schema to have a self-contained response which may still
    // needs access to the schema strings
    pub(crate) schema: Arc<Schema>,
    pub(crate) schema_version: SchemaVersion,
    pub(crate) runtime: R,
    pub(crate) operation_metrics: GraphqlOperationMetrics,
    auth: AuthService,
    retry_budgets: RetryBudgets,
    operation_cache: <R::OperationCacheFactory as OperationCacheFactory>::Cache<Arc<PreparedOperation>>,
}

impl<R: Runtime> Engine<R> {
    /// schema_version is used in operation cache key which ensures we only retrieve cached
    /// operation for the same schema version. If none is provided, a random one is generated.
    pub async fn new(schema: Arc<Schema>, schema_version: Option<&[u8]>, runtime: R) -> Self {
        let auth = gateway_v2_auth::AuthService::new_v2(
            schema.settings.auth_config.clone().unwrap_or_default(),
            runtime.kv().clone(),
        );

        Self {
            schema_version: SchemaVersion({
                let mut out = Vec::new();
                match schema_version {
                    Some(version) => {
                        out.push(0x00);
                        out.extend_from_slice(version);
                    }
                    None => {
                        out.push(0x01);
                        out.extend_from_slice(&ulid::Ulid::new().to_bytes());
                    }
                }
                out
            }),
            auth,
            retry_budgets: RetryBudgets::build(&schema),
            operation_metrics: GraphqlOperationMetrics::build(runtime.meter()),
            operation_cache: runtime.operation_cache_factory().create().await,
            schema,
            runtime,
        }
    }

    pub async fn execute<F>(self: &Arc<Self>, request: http::Request<F>) -> http::Response<Body>
    where
        F: Future<Output = Result<Bytes, (http::StatusCode, String)>> + Send,
    {
        // Following the recommendation of the GraphQL over HTTP spec to require a valid Accept
        // header
        let (parts, body) = request.into_parts();
        let Some(format) = ResponseFormat::extract_from(&parts.headers) else {
            // GraphQL-over-HTTP spec:
            //   In alignment with the HTTP 1.1 Accept specification, when a client does not include at least one supported media type in the Accept HTTP header, the server MUST either:
            //     1. Respond with a 406 Not Acceptable status code and stop processing the request (RECOMMENDED); OR
            //     2. Disregard the Accept header and respond with the server's choice of media type (NOT RECOMMENDED).
            return Http::from(
                ResponseFormat::application_json(), // assumed default for the error response
                RefusedRequestResponse::not_acceptable_error(),
            );
        };

        if parts.method == http::Method::POST {
            // GraphQL-over-HTTP spec:
            //   If the client does not supply a Content-Type header with a POST request,
            //   the server SHOULD reject the request using the appropriate 4xx status code.
            if !content_type_is_application_json(&parts.headers) {
                return Http::from(format, RefusedRequestResponse::unsupported_media_type());
            }
        } else if parts.method != http::Method::GET {
            return Http::from(
                format,
                RefusedRequestResponse::method_not_allowed("Only GET or POST are supported."),
            );
        }

        let request_context_fut = self
            .create_request_context(!parts.method.is_safe(), parts.headers, format)
            .map_err(|response| Http::from(format, response));

        let graphql_request_fut = async move {
            if parts.method == http::Method::POST {
                body.await
                    .map_err(|(status, message)| {
                        Http::from(
                            format,
                            Response::refuse_request_with(status, GraphqlError::new(message, ErrorCode::BadRequest)),
                        )
                    })
                    .and_then(|body| {
                        serde_json::from_slice(&body).map_err(|err| {
                            Http::from(
                                format,
                                RefusedRequestResponse::not_well_formed_graphql_over_http_request(&err.to_string()),
                            )
                        })
                    })
            } else {
                let query = parts.uri.query().unwrap_or_default();
                serde_urlencoded::from_str::<QueryParamsRequest>(query)
                    .map(|request| BatchRequest::Single(request.into()))
                    .map_err(|err| {
                        Http::from(
                            format,
                            RefusedRequestResponse::not_well_formed_graphql_over_http_request(&err.to_string()),
                        )
                    })
            }
        };

        // Retrieve the request body while processing the headers
        match futures::try_join!(request_context_fut, graphql_request_fut) {
            Ok((request_context, request)) => self.execute_well_formed_graphql_request(request_context, request).await,
            Err(response) => response,
        }
    }

    pub async fn create_websocket_session(
        self: &Arc<Self>,
        headers: http::HeaderMap,
    ) -> Result<WebsocketSession<R>, Cow<'static, str>> {
        let response_format = ResponseFormat::Streaming(StreamingResponseFormat::GraphQLOverWebSocket);

        let request_context = match self.create_request_context(true, headers, response_format).await {
            Ok(context) => context,
            Err(response) => {
                return Err(response
                    .errors()
                    .first()
                    .map(|error| error.message.clone())
                    .unwrap_or("Internal server error".into()))
            }
        };

        Ok(WebsocketSession {
            engine: Arc::clone(self),
            request_context: Arc::new(request_context),
        })
    }

    async fn create_request_context(
        &self,
        mutations_allowed: bool,
        headers: http::HeaderMap,
        response_format: ResponseFormat,
    ) -> Result<RequestContext<<R::Hooks as Hooks>::Context>, Response> {
        let client = Client::extract_from(&headers);

        let (hooks_context, headers) = self
            .runtime
            .hooks()
            .on_gateway_request(headers)
            .await
            .map_err(|ErrorResponse { status, error }| Response::refuse_request_with(status, error))?;

        let Some(access_token) = self.auth.authenticate(&headers).await else {
            return Err(RefusedRequestResponse::unauthenticated());
        };

        // Currently it doesn't rely on authentication, but likely will at some point.
        if self.runtime.rate_limiter().limit(&RateLimitKey::Global).await.is_err() {
            return Err(RefusedRequestResponse::gateway_rate_limited());
        }

        Ok(RequestContext {
            mutations_allowed,
            headers,
            response_format,
            client,
            access_token,
            hooks_context,
        })
    }

    async fn execute_well_formed_graphql_request(
        self: &Arc<Self>,
        request_context: RequestContext<<R::Hooks as Hooks>::Context>,
        request: BatchRequest,
    ) -> http::Response<Body> {
        match request {
            BatchRequest::Single(request) => {
                if let ResponseFormat::Streaming(format) = request_context.response_format {
                    Http::stream(format, self.execute_stream(Arc::new(request_context), request)).await
                } else {
                    let Some(response) = self
                        .runtime
                        .with_timeout(
                            self.schema.settings.timeout,
                            self.execute_single(&request_context, request),
                        )
                        .await
                    else {
                        return Http::from(request_context.response_format, RequestErrorResponse::gateway_timeout());
                    };
                    Http::from(request_context.response_format, response)
                }
            }
            BatchRequest::Batch(requests) => {
                let ResponseFormat::Complete(format) = request_context.response_format else {
                    return Http::from(
                        request_context.response_format,
                        RequestErrorResponse::bad_request_but_well_formed_graphql_over_http_request(
                            "batch requests cannot be returned as multipart or event-stream responses",
                        ),
                    );
                };
                let Some(responses) = self
                    .runtime
                    .with_timeout(
                        self.schema.settings.timeout,
                        futures_util::stream::iter(requests.into_iter())
                            .then(|request| self.execute_single(&request_context, request))
                            .collect::<Vec<_>>(),
                    )
                    .await
                else {
                    return Http::from(request_context.response_format, RequestErrorResponse::gateway_timeout());
                };
                Http::batch(format, responses)
            }
        }
    }

    async fn execute_single(
        &self,
        request_context: &RequestContext<<R::Hooks as Hooks>::Context>,
        request: Request,
    ) -> Response {
        let start = Instant::now();
        let span = GqlRequestSpan::create();
        async {
            let ctx = PreExecutionContext::new(self, request_context);
            let (operation_metrics_attributes, response) = ctx.execute_single(request).await;
            let elapsed = start.elapsed();
            let status = response.graphql_status();

            if let Some(operation_metrics_attributes) = operation_metrics_attributes {
                span.record_gql_request((&operation_metrics_attributes).into());

                self.operation_metrics.record_operation_duration(
                    GraphqlRequestMetricsAttributes {
                        operation: operation_metrics_attributes,
                        status,
                        cache_status: None,
                        client: request_context.client.clone(),
                    },
                    elapsed,
                );
            }

            span.record_gql_status(status);

            if status.is_success() {
                tracing::debug!(target: GRAFBASE_TARGET, "gateway request")
            } else {
                let message = response
                    .errors()
                    .first()
                    .map(|error| error.message.clone())
                    .unwrap_or_else(|| String::from("gateway error").into());

                tracing::debug!(target: GRAFBASE_TARGET, "{message}")
            }

            response
        }
        .instrument(span.clone())
        .await
    }

    fn execute_stream(
        self: &Arc<Self>,
        request_context: Arc<RequestContext<<R::Hooks as Hooks>::Context>>,
        request: Request,
    ) -> impl Stream<Item = Response> + Send + 'static {
        let start = Instant::now();
        let engine = Arc::clone(self);
        let (sender, receiver) = mpsc::channel(2);

        let span = GqlRequestSpan::create();
        let span_clone = span.clone();
        receiver.join(
            async move {
                let ctx = PreExecutionContext::new(&engine, &request_context);
                let (operation_metrics_attributes, status) = ctx.execute_stream(request, sender).await;
                let elapsed = start.elapsed();

                if let Some(operation_metrics_attributes) = operation_metrics_attributes {
                    tracing::Span::current().record_gql_request((&operation_metrics_attributes).into());

                    engine.operation_metrics.record_operation_duration(
                        GraphqlRequestMetricsAttributes {
                            operation: operation_metrics_attributes,
                            status,
                            cache_status: None,
                            client: request_context.client.clone(),
                        },
                        elapsed,
                    );
                }

                span.record_gql_status(status);

                if status.is_success() {
                    tracing::debug!(target: GRAFBASE_TARGET, "gateway request")
                } else {
                    tracing::debug!(target: GRAFBASE_TARGET, "gateway error")
                }
            }
            .instrument(span_clone),
        )
    }
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    async fn execute_single(mut self, request: Request) -> (Option<OperationMetricsAttributes>, Response) {
        let operation_plan = match self.prepare_operation(request).await {
            Ok(operation_plan) => operation_plan,
            Err((metadata, response)) => return (metadata, response),
        };

        let metrics_attributes = Some(operation_plan.metrics_attributes.clone());
        let response = if matches!(operation_plan.ty(), OperationType::Subscription) {
            RequestErrorResponse::bad_request_but_well_formed_graphql_over_http_request(
                "Subscriptions are only suported on streaming transports. Try making a request with SSE or WebSockets",
            )
        } else {
            self.execute_query_or_mutation(operation_plan).await
        };

        (metrics_attributes, response)
    }

    async fn execute_stream(
        mut self,
        request: Request,
        mut sender: mpsc::Sender<Response>,
    ) -> (Option<OperationMetricsAttributes>, GraphqlResponseStatus) {
        // If it's a subscription, we at least have a timeout on the operation preparation.
        let result = self
            .engine
            .runtime
            .with_timeout(self.engine.schema.settings.timeout, async {
                let operation_plan = match self.prepare_operation(request).await {
                    Ok(operation_plan) => operation_plan,
                    Err((metadata, response)) => {
                        let status = response.graphql_status();
                        sender.send(response).await.ok();
                        return Err((metadata, status));
                    }
                };
                if matches!(operation_plan.ty(), OperationType::Query | OperationType::Mutation) {
                    let metrics_attributes = Some(operation_plan.metrics_attributes.clone());
                    let response = self.execute_query_or_mutation(operation_plan).await;
                    let status = response.graphql_status();
                    sender.send(response).await.ok();
                    Err((metrics_attributes, status))
                } else {
                    Ok((self, operation_plan))
                }
            })
            .await;

        let (ctx, operation_plan) = match result {
            Some(Ok((ctx, operation_plan))) => (ctx, operation_plan),
            Some(Err((metadata, status))) => return (metadata, status),
            None => {
                let response = RequestErrorResponse::gateway_timeout();
                let status = response.graphql_status();
                sender.send(response).await.ok();
                return (None, status);
            }
        };

        let mut status: GraphqlResponseStatus = GraphqlResponseStatus::Success;
        struct Sender<'a> {
            sender: mpsc::Sender<Response>,
            status: &'a mut GraphqlResponseStatus,
        }

        impl crate::execution::ResponseSender for Sender<'_> {
            type Error = mpsc::SendError;
            async fn send(&mut self, response: Response) -> Result<(), Self::Error> {
                *self.status = self.status.union(response.graphql_status());
                self.sender.send(response).await
            }
        }

        let metrics_attributes = Some(operation_plan.metrics_attributes.clone());
        ctx.execute_subscription(
            operation_plan,
            Sender {
                sender,
                status: &mut status,
            },
        )
        .await;
        (metrics_attributes, status)
    }

    async fn prepare_operation(
        &mut self,
        request: Request,
    ) -> Result<ExecutableOperation, (Option<OperationMetricsAttributes>, Response)> {
        let start = SystemTime::now();
        let result = self.prepare_operation_inner(request).await;
        let duration = SystemTime::now().duration_since(start).unwrap_or_default();

        match result {
            Ok(operation) => {
                let attributes = QueryPreparationAttributes {
                    operation_name: operation.prepared.metrics_attributes.name.clone(),
                    document: Some(operation.prepared.metrics_attributes.sanitized_query.clone()),
                    success: true,
                };

                self.operation_metrics.record_preparation_latency(attributes, duration);

                Ok(operation)
            }
            Err(e) => {
                dbg!(1);
                let attributes = QueryPreparationAttributes {
                    operation_name: None,
                    document: None,
                    success: false,
                };

                self.operation_metrics.record_preparation_latency(attributes, duration);

                Err(e)
            }
        }
    }

    async fn prepare_operation_inner(
        &mut self,
        request: Request,
    ) -> Result<ExecutableOperation, (Option<OperationMetricsAttributes>, Response)> {
        let result = {
            let OperationDocument { cache_key, load_fut } = match self.determine_operation_document(&request) {
                Ok(doc) => doc,
                // If we have an error a this stage, it means we couldn't determine what document
                // to load, so we don't consider it a well-formed GraphQL-over-HTTP request.
                Err(err) => return Err((None, Response::refuse_request_with(http::StatusCode::BAD_REQUEST, err))),
            };

            if let Some(operation) = self.operation_cache.get(&cache_key).await {
                self.engine.operation_metrics.record_operation_cache_hit();
                Ok(operation)
            } else {
                self.engine.operation_metrics.record_operation_cache_miss();
                match load_fut.await {
                    Ok(document) => Err((cache_key, document)),
                    Err(err) => return Err((None, Response::request_error([err]))),
                }
            }
        };

        let operation = match result {
            Ok(operation) => operation,
            Err((cache_key, document)) => {
                let operation = Operation::build(&self.schema, &request, &document)
                    .map(Arc::new)
                    .map_err(|mut err| (err.take_metrics_attributes(), Response::request_error([err])))?;

                let cache_fut = self.engine.operation_cache.insert(cache_key, operation.clone());
                self.push_background_future(cache_fut.boxed());

                operation
            }
        };

        // GraphQL-over-HTTP spec:
        //   GET requests MUST NOT be used for executing mutation operations. If the values of {query} and {operationName} indicate that
        //   a mutation operation is to be executed, the server MUST respond with error status code 405 (Method Not Allowed) and halt
        //   execution. This restriction is necessary to conform with the long-established semantics of safe methods within HTTP.
        //
        // While it's technically a RequestError at this stage, as we have a well-formed GraphQL-over-HTTP request,
        // we must return a 4xx without regard to the Accept header in all cases, so it's akin to denying the request.
        //
        // This error would be confusing for a websocket connection, but today mutation are always
        // allowed for it.
        if operation.ty.is_mutation() && !self.request_context.mutations_allowed {
            return Err((
                Some(operation.metrics_attributes.clone()),
                RefusedRequestResponse::method_not_allowed("Mutation is not allowed with a safe method like GET"),
            ));
        }

        let variables = Variables::build(self.schema.as_ref(), &operation, request.variables).map_err(|errors| {
            (
                Some(operation.metrics_attributes.clone()),
                Response::request_error(errors),
            )
        })?;

        self.finalize_operation(Arc::clone(&operation), variables)
            .await
            .map_err(|err| {
                (
                    Some(operation.metrics_attributes.clone()),
                    Response::request_error([err]),
                )
            })
    }
}

pub struct WebsocketSession<R: Runtime> {
    engine: Arc<Engine<R>>,
    request_context: Arc<RequestContext<<R::Hooks as Hooks>::Context>>,
}

impl<R: Runtime> Clone for WebsocketSession<R> {
    fn clone(&self) -> Self {
        Self {
            engine: Arc::clone(&self.engine),
            request_context: Arc::clone(&self.request_context),
        }
    }
}

pub(crate) struct RequestContext<C> {
    pub mutations_allowed: bool,
    pub headers: http::HeaderMap,
    pub response_format: ResponseFormat,
    pub client: Option<Client>,
    pub access_token: AccessToken,
    pub hooks_context: C,
}

impl<R: Runtime> WebsocketSession<R> {
    pub fn execute(&self, event: websocket::SubscribeEvent) -> impl Stream<Item = websocket::Message> {
        let websocket::SubscribeEvent { id, payload } = event;
        self.engine
            .execute_stream(self.request_context.clone(), payload.0)
            .map(move |response| match response {
                Response::RefusedRequest(_) => websocket::Message::Error {
                    id: id.clone(),
                    payload: websocket::ResponsePayload(response),
                },
                response => websocket::Message::Next {
                    id: id.clone(),
                    payload: websocket::ResponsePayload(response),
                },
            })
    }
}

fn content_type_is_application_json(headers: &http::HeaderMap) -> bool {
    static APPLICATION_JSON: http::HeaderValue = http::HeaderValue::from_static("application/json");

    let Some(header) = headers.get(http::header::CONTENT_TYPE) else {
        return false;
    };

    let header = header.to_str().unwrap_or_default();
    let (without_parameters, _) = header.split_once(';').unwrap_or((header, ""));

    without_parameters == APPLICATION_JSON
}
