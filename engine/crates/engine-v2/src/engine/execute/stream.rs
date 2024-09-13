use std::sync::Arc;

use async_runtime::stream::StreamExt as _;
use engine_parser::types::OperationType;
use enumset::EnumSet;
use futures::{
    channel::{mpsc, oneshot},
    stream::BoxStream,
    StreamExt as _,
};
use futures_util::SinkExt;
use grafbase_telemetry::{
    grafbase_client::Client,
    graphql::{GraphqlExecutionTelemetry, GraphqlOperationAttributes, GraphqlResponseStatus},
    metrics::{EngineMetrics, GraphqlErrorAttributes, GraphqlRequestMetricsAttributes},
    span::graphql::GraphqlOperationSpan,
};
use tracing::Instrument;
use web_time::Instant;

use crate::{
    engine::{HooksContext, RequestContext, RuntimeExt},
    execution::{PreExecutionContext, ResponseSender},
    request::Request,
    response::Response,
    Engine, ErrorCode, Runtime,
};

pub(crate) struct StreamResponse {
    pub stream: BoxStream<'static, Response>,
    pub telemetry: oneshot::Receiver<GraphqlExecutionTelemetry<ErrorCode>>,
    pub on_operation_response_outputs: mpsc::Receiver<Vec<u8>>,
}

impl<R: Runtime> Engine<R> {
    pub(super) fn execute_stream(
        self: &Arc<Self>,
        request_context: Arc<RequestContext>,
        hooks_context: HooksContext<R>,
        request: Request,
    ) -> StreamResponse {
        let start = Instant::now();
        let engine = Arc::clone(self);
        let (response_sender, response_receiver) = mpsc::channel(2);

        let graphql_span = GraphqlOperationSpan::default();
        let span = graphql_span.span.clone();

        let (telemetry_sender, telemetry_receiver) = oneshot::channel();
        let (on_operation_response_outputs_sender, on_operation_response_outputs_receiver) = mpsc::channel(2);

        let stream = response_receiver
            .join(
                async move {
                    let ctx = PreExecutionContext::new(&engine, &request_context, hooks_context);
                    let mut errors_count = 0;
                    let mut distinct_error_codes = EnumSet::new();

                    struct Sender<'a> {
                        errors_count: &'a mut usize,
                        distinct_error_codes: &'a mut EnumSet<ErrorCode>,
                        on_operation_response_outputs_sender: mpsc::Sender<Vec<u8>>,
                        response_sender: mpsc::Sender<Response>,
                    }

                    impl ResponseSender for Sender<'_> {
                        type Error = mpsc::SendError;
                        async fn send(&mut self, mut response: Response) -> Result<(), Self::Error> {
                            *self.errors_count += response.errors().len();
                            *self.distinct_error_codes |= response.distinct_error_codes();
                            if let Some(output) = response.take_on_operation_response_output() {
                                // If the receiver is dropped we don't really care.
                                let _ = self.on_operation_response_outputs_sender.try_send(output);
                            }
                            self.response_sender.send(response).await
                        }
                    }

                    let (operation_attributes, status) = ctx
                        .execute_stream(
                            request,
                            Sender {
                                errors_count: &mut errors_count,
                                distinct_error_codes: &mut distinct_error_codes,
                                on_operation_response_outputs_sender,
                                response_sender,
                            },
                        )
                        .await;

                    let mut telemetry = GraphqlExecutionTelemetry {
                        operations: Vec::new(),
                        errors_count: errors_count as u64,
                        distinct_error_codes: distinct_error_codes.into_iter().collect(),
                    };
                    if let Some(operation) = operation_attributes {
                        telemetry.operations.push((operation.ty, operation.name.clone()));
                        graphql_span.record_operation(&operation);

                        engine.runtime.metrics().record_operation_duration(
                            GraphqlRequestMetricsAttributes {
                                operation,
                                status,
                                cache_status: None,
                                client: request_context.client.clone(),
                            },
                            start.elapsed(),
                        );
                    }
                    graphql_span.record_response_status(status);
                    graphql_span.record_distinct_error_codes(telemetry.distinct_error_codes.as_slice());
                    let _ = telemetry_sender.send(telemetry);

                    // After recording all operation metadata
                    tracing::debug!("Executed operation in stream.")
                }
                .instrument(span),
            )
            .boxed();

        StreamResponse {
            stream,
            telemetry: telemetry_receiver,
            on_operation_response_outputs: on_operation_response_outputs_receiver,
        }
    }
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    async fn execute_stream<S>(
        mut self,
        request: Request,
        mut sender: S,
    ) -> (Option<GraphqlOperationAttributes>, GraphqlResponseStatus)
    where
        S: ResponseSender<Error = mpsc::SendError>,
    {
        let engine = self.engine;
        let client = &self.request_context.client;

        // If it's a subscription, we at least have a timeout on the operation preparation.
        let result = engine
            .runtime
            .with_timeout(self.engine.schema.settings.timeout, async {
                let operation = match self.prepare_operation(request).await {
                    Ok(operation_plan) => operation_plan,
                    Err(response) => {
                        let attributes = response.operation_attributes().cloned();
                        let status = response.graphql_status();
                        sender.send(response).await.ok();
                        return Err((attributes, status));
                    }
                };

                if matches!(operation.ty(), OperationType::Query | OperationType::Mutation) {
                    let attributes = operation.attributes.clone();
                    let response = self.execute_query_or_mutation(operation).await;
                    let status = response.graphql_status();

                    sender.send(response).await.ok();

                    Err((Some(attributes), status))
                } else {
                    Ok((self, operation))
                }
            })
            .await;

        let (ctx, operation) = match result {
            Some(Ok((ctx, operation))) => (ctx, operation),
            Some(Err((metadata, status))) => return (metadata, status),
            None => {
                let response = Response::gateway_timeout();
                let status = response.graphql_status();
                sender.send(response).await.ok();
                return (None, status);
            }
        };

        let mut status: GraphqlResponseStatus = GraphqlResponseStatus::Success;

        struct Sender<'a, S> {
            inner: S,
            status: &'a mut GraphqlResponseStatus,
            operation_name: Option<String>,
            client: &'a Option<Client>,
            metrics: &'a EngineMetrics,
        }

        impl<S: ResponseSender<Error = mpsc::SendError>> ResponseSender for Sender<'_, S> {
            type Error = mpsc::SendError;
            async fn send(&mut self, response: Response) -> Result<(), Self::Error> {
                *self.status = self.status.union(response.graphql_status());

                for error_code in response.distinct_error_codes() {
                    self.metrics.increment_graphql_errors(GraphqlErrorAttributes {
                        code: error_code.into(),
                        operation_name: self.operation_name.clone(),
                        client: self.client.clone(),
                    })
                }

                self.inner.send(response).await
            }
        }

        let operation_name = operation.attributes.name.original().map(str::to_string);
        let attributes = operation.attributes.clone();
        ctx.execute_subscription(
            operation,
            Sender {
                inner: sender,
                status: &mut status,
                operation_name,
                client,
                metrics: engine.runtime.metrics(),
            },
        )
        .await;

        (Some(attributes), status)
    }
}
