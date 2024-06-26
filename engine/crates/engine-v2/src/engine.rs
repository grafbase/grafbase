use std::{borrow::Cow, sync::Arc};
use web_time::Instant;

use ::runtime::{auth::AccessToken, hooks::Hooks};
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
use schema::Schema;
use tracing::Instrument;

use crate::{
    execution::{ExecutionContext, ExecutionCoordinator},
    http_response::{HttpGraphqlResponse, OperationMetadata},
    operation::{Operation, Variables},
    plan::OperationPlan,
    response::{GraphqlError, Response},
    websocket,
};

mod runtime;
mod trusted_documents;

pub use runtime::Runtime;

pub struct Engine<R> {
    // We use an Arc for the schema to have a self-contained response which may still
    // needs access to the schema strings
    pub(crate) schema: Arc<Schema>,
    pub(crate) runtime: R,
    operation_metrics: GraphqlOperationMetrics,
    auth: AuthService,
}

impl<R: Runtime> Engine<R> {
    pub fn new(schema: Arc<Schema>, runtime: R) -> Self {
        let auth = gateway_v2_auth::AuthService::new_v2(
            schema.settings.auth_config.clone().unwrap_or_default(),
            runtime.kv().clone(),
        );

        Self {
            schema,
            auth,
            operation_metrics: GraphqlOperationMetrics::build(runtime.meter()),
            runtime,
        }
    }

    pub async fn execute(
        self: &Arc<Self>,
        headers: http::HeaderMap,
        batch_request: BatchRequest,
    ) -> HttpGraphqlResponse {
        let (hooks_context, headers) = match self.runtime.hooks().on_gateway_request(headers).await {
            Ok(result) => result,
            Err(error) => return Response::execution_error(error).into(),
        };

        if let Some(access_token) = self.auth.authorize(&headers).await {
            let context = RequestContext::new(headers, access_token, hooks_context);
            self.execute_with_access_token(context, batch_request).await
        } else if let Some(streaming_format) = headers.typed_get::<StreamingFormat>() {
            HttpGraphqlResponse::stream_request_error(streaming_format, "Unauthorized")
        } else {
            HttpGraphqlResponse::request_error("Unauthorized")
        }
    }

    pub async fn create_session(self: &Arc<Self>, headers: http::HeaderMap) -> Result<Session<R>, Cow<'static, str>> {
        let (hooks_context, headers) = match self.runtime.hooks().on_gateway_request(headers).await {
            Ok(result) => result,
            Err(error) => return Err(Cow::from(error.to_string())),
        };

        match self.auth.authorize(&headers).await {
            Some(access_token) => Ok(Session {
                engine: Arc::clone(self),
                request_context: Arc::new(RequestContext::new(headers, access_token, hooks_context)),
            }),
            None => Err(Cow::from("Forbidden")),
        }
    }

    async fn execute_with_access_token(
        self: &Arc<Self>,
        request_context: RequestContext<<R::Hooks as Hooks>::Context>,
        batch_request: BatchRequest,
    ) -> HttpGraphqlResponse {
        let streaming_format = request_context.headers.typed_get::<StreamingFormat>();
        match batch_request {
            BatchRequest::Single(request) => {
                if let Some(streaming_format) = streaming_format {
                    convert_stream_to_http_response(
                        streaming_format,
                        self.execute_stream(Arc::new(request_context), request),
                    )
                    .await
                } else {
                    self.execute_single(&request_context, request).await
                }
            }
            BatchRequest::Batch(requests) => {
                if streaming_format.is_some() {
                    return HttpGraphqlResponse::request_error(
                        "batch requests can't use multipart or event-stream responses",
                    );
                }
                HttpGraphqlResponse::batch_response(
                    futures_util::stream::iter(requests.into_iter())
                        .then(|request| self.execute_single(&request_context, request))
                        .collect::<Vec<_>>()
                        .await,
                )
            }
        }
    }

    async fn execute_single(
        &self,
        request_context: &RequestContext<<R::Hooks as Hooks>::Context>,
        request: Request,
    ) -> HttpGraphqlResponse {
        let start = Instant::now();
        let span = GqlRequestSpan::new().into_span();
        async {
            let ctx = ExecutionContext {
                engine: self,
                request_context,
            };
            let (operation_attributes, response) = ctx.execute_single(request).await;
            let status = response.status();
            let mut metadata = OperationMetadata {
                operation_name: None,
                operation_type: None,
                has_errors: !status.is_success(),
            };
            if let Some(mut attrs) = operation_attributes {
                span.record_gql_request(GqlRequestAttributes {
                    operation_type: attrs.ty,
                    operation_name: attrs.name.clone(),
                });
                metadata.operation_name.clone_from(&attrs.name);
                metadata.operation_type = Some(attrs.ty);
                attrs.status = status;
                self.operation_metrics.record(attrs, start.elapsed());
            }
            span.record_gql_status(status);
            HttpGraphqlResponse::from(response).with_metadata(metadata)
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

        let span = GqlRequestSpan::new().into_span();
        let span_clone = span.clone();
        receiver.join(
            async move {
                let ctx = ExecutionContext {
                    engine: &engine,
                    request_context: &request_context,
                };
                let (operation_attributes, status) = ctx.execute_stream(request, sender).await;
                if let Some(mut attrs) = operation_attributes {
                    span.record_gql_request(GqlRequestAttributes {
                        operation_type: attrs.ty,
                        operation_name: attrs.name.clone(),
                    });
                    attrs.status = status;
                    engine.operation_metrics.record(attrs, start.elapsed());
                }

                span.record_gql_status(status);
            }
            .instrument(span_clone),
        )
    }
}

async fn convert_stream_to_http_response(
    streaming_format: StreamingFormat,
    stream: impl Stream<Item = Response> + Send + 'static,
) -> HttpGraphqlResponse {
    let mut stream = Box::pin(stream);
    let Some(first_response) = stream.next().await else {
        return HttpGraphqlResponse::request_error("Empty stream");
    };
    HttpGraphqlResponse::from_stream(
        streaming_format,
        // Not perfect for the errors count, but good enough to detect a request error
        first_response.status(),
        futures_util::stream::iter(std::iter::once(first_response)).chain(stream),
    )
}

impl<'ctx, R: Runtime> ExecutionContext<'ctx, R> {
    async fn execute_single(self, mut request: Request) -> (Option<GraphqlOperationMetricsAttributes>, Response) {
        if let Err(err) = self.handle_persisted_query(&mut request).await {
            return (None, Response::bad_request(err));
        }
        let (operation, operation_attributes) = match Operation::build(&self.schema, self.request_context, &request) {
            Ok(res) => res,
            Err(mut err) => {
                return (err.take_operation_attributes(), Response::bad_request(err));
            }
        };

        let response = if matches!(operation.ty, OperationType::Subscription) {
            Response::bad_request(GraphqlError::new(
                "Subscriptions are only suported on streaming transports. Try making a request with SSE or WebSockets",
            ))
        } else {
            match self.prepare_coordinator(operation, request.variables).await {
                Ok(coordinator) => coordinator.execute().await,
                Err(errors) => Response::bad_request_from_errors(errors),
            }
        };

        (operation_attributes, response)
    }

    async fn execute_stream(
        self,
        mut request: Request,
        mut sender: mpsc::Sender<Response>,
    ) -> (Option<GraphqlOperationMetricsAttributes>, GraphqlResponseStatus) {
        if let Err(err) = self.handle_persisted_query(&mut request).await {
            let response = Response::bad_request(err);
            let status = response.status();
            sender.send(response).await.ok();
            return (None, status);
        }
        let (operation, operation_attributes) = match Operation::build(&self.schema, self.request_context, &request) {
            Ok(res) => res,
            Err(mut err) => {
                let attrs = err.take_operation_attributes();
                let response = Response::bad_request(err);
                let status = response.status();
                sender.send(response).await.ok();
                return (attrs, status);
            }
        };

        let coordinator = match self.prepare_coordinator(operation, request.variables).await {
            Ok(coordinator) => coordinator,
            Err(errors) => {
                let response = Response::bad_request_from_errors(errors);
                let status = response.status();
                sender.send(response).await.ok();
                return (operation_attributes, status);
            }
        };

        if matches!(
            coordinator.operation().ty,
            OperationType::Query | OperationType::Mutation
        ) {
            let response = coordinator.execute().await;
            let status = response.status();
            sender.send(response).await.ok();
            return (operation_attributes, status);
        }

        let mut status: GraphqlResponseStatus = GraphqlResponseStatus::Success;
        struct Sender<'a> {
            sender: mpsc::Sender<Response>,
            status: &'a mut GraphqlResponseStatus,
        }

        impl crate::execution::ResponseSender for Sender<'_> {
            type Error = mpsc::SendError;
            async fn send(&mut self, response: Response) -> Result<(), Self::Error> {
                *self.status = self.status.union(response.status());
                self.sender.send(response).await
            }
        }

        coordinator
            .execute_subscription(Sender {
                sender,
                status: &mut status,
            })
            .await;
        (operation_attributes, status)
    }

    async fn prepare_coordinator(
        self,
        operation: Operation,
        variables: engine::Variables,
    ) -> Result<ExecutionCoordinator<'ctx, R>, Vec<GraphqlError>> {
        let variables = Variables::build(self.schema.as_ref(), &operation, variables)
            .map_err(|errors| errors.into_iter().map(Into::into).collect::<Vec<_>>())?;

        let operation_plan = Arc::new(
            OperationPlan::build(self, &variables, operation)
                .await
                .map_err(|err| vec![err.into()])?,
        );

        Ok(ExecutionCoordinator::new(self, operation_plan, variables))
    }
}

pub struct Session<R: Runtime> {
    engine: Arc<Engine<R>>,
    request_context: Arc<RequestContext<<R::Hooks as Hooks>::Context>>,
}

impl<R: Runtime> Clone for Session<R> {
    fn clone(&self) -> Self {
        Self {
            engine: Arc::clone(&self.engine),
            request_context: Arc::clone(&self.request_context),
        }
    }
}

pub(crate) struct RequestContext<C> {
    pub headers: http::HeaderMap,
    pub client: Option<Client>,
    pub access_token: AccessToken,
    pub hooks_context: C,
}

impl<C> RequestContext<C> {
    fn new(headers: http::HeaderMap, access_token: AccessToken, hooks_context: C) -> Self {
        let client = Client::extract_from(&headers);

        Self {
            headers,
            client,
            access_token,
            hooks_context,
        }
    }
}

impl<R: Runtime> Session<R> {
    pub fn execute_websocket(&self, id: String, request: Request) -> impl Stream<Item = websocket::Message> {
        self.engine
            .execute_stream(self.request_context.clone(), request)
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
