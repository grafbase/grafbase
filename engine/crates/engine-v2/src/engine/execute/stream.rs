use std::sync::Arc;

use async_runtime::stream::StreamExt as _;
use engine_parser::types::OperationType;
use futures::{
    channel::{mpsc, oneshot},
    stream::BoxStream,
    StreamExt as _,
};
use futures_util::SinkExt;
use grafbase_telemetry::{
    graphql::{GraphqlExecutionTelemetry, GraphqlOperationAttributes, GraphqlResponseStatus},
    metrics::{GraphqlErrorAttributes, GraphqlRequestMetricsAttributes},
    span::graphql::GraphqlOperationSpan,
};
use runtime::hooks::Hooks;
use tracing::Instrument;
use web_time::Instant;

use crate::{
    engine::{errors, HooksContext, RequestContext},
    execution::{PreExecutionContext, ResponseSender},
    request::Request,
    response::{ErrorCode, ErrorCodeCounter, Response},
    Engine, Runtime,
};

pub(crate) struct StreamResponse<OnOperationResponseOutput> {
    pub stream: BoxStream<'static, Response<OnOperationResponseOutput>>,
    pub telemetry: oneshot::Receiver<GraphqlExecutionTelemetry<ErrorCode>>,
}

impl<R: Runtime> Engine<R> {
    pub(super) fn execute_stream(
        self: &Arc<Self>,
        request_context: Arc<RequestContext>,
        hooks_context: HooksContext<R>,
        request: Request,
    ) -> StreamResponse<<R::Hooks as Hooks>::OnOperationResponseOutput> {
        let start = Instant::now();
        let engine = Arc::clone(self);
        let (response_sender, response_receiver) = mpsc::channel(2);

        let graphql_span = GraphqlOperationSpan::default();
        let span = graphql_span.span.clone();

        let (telemetry_sender, telemetry_receiver) = oneshot::channel();

        let stream = response_receiver
            .join(
                async move {
                    let ctx = PreExecutionContext::new(&engine, &request_context, hooks_context);
                    let mut status = GraphqlResponseStatus::Success;
                    let mut error_code_counter = ErrorCodeCounter::default();

                    struct Sender<'a, O> {
                        status: &'a mut GraphqlResponseStatus,
                        error_code_counter: &'a mut ErrorCodeCounter,
                        response_sender: mpsc::Sender<Response<O>>,
                    }

                    impl<O: Send + 'static> ResponseSender<O> for Sender<'_, O> {
                        type Error = mpsc::SendError;
                        async fn send(&mut self, response: Response<O>) -> Result<(), Self::Error> {
                            *self.status = self.status.union(response.graphql_status());
                            self.error_code_counter.add(response.error_code_counter());
                            self.response_sender.send(response).await
                        }
                    }

                    let operation_attributes = ctx
                        .execute_stream(
                            request,
                            Sender {
                                status: &mut status,
                                error_code_counter: &mut error_code_counter,
                                response_sender,
                            },
                        )
                        .await;

                    let mut telemetry = GraphqlExecutionTelemetry {
                        operations: Vec::new(),
                        errors_count_by_code: error_code_counter.to_vec(),
                    };
                    if let Some(operation) = operation_attributes {
                        telemetry.operations.push((operation.ty, operation.name.clone()));
                        graphql_span.record_operation(&operation);

                        for (error_code, _) in &telemetry.errors_count_by_code {
                            engine
                                .runtime
                                .metrics()
                                .increment_graphql_errors(GraphqlErrorAttributes {
                                    code: error_code.into(),
                                    operation_name: operation.name.original().map(str::to_string),
                                    client: request_context.client.clone(),
                                });
                        }

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
                    graphql_span.record_response(status, &telemetry.errors_count_by_code);
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
        }
    }
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    async fn execute_stream<S>(mut self, request: Request, mut sender: S) -> Option<GraphqlOperationAttributes>
    where
        S: ResponseSender<<R::Hooks as Hooks>::OnOperationResponseOutput, Error = mpsc::SendError>,
    {
        // If it's a subscription, we at least have a timeout on the operation preparation.
        let result = self
            .engine
            .with_gateway_timeout(async {
                let operation = match self.prepare_operation(request).await {
                    Ok(operation_plan) => operation_plan,
                    Err(response) => {
                        let attributes = response.operation_attributes().cloned();
                        sender.send(response).await.ok();
                        return Err(attributes);
                    }
                };

                if matches!(operation.ty(), OperationType::Query | OperationType::Mutation) {
                    let attributes = operation.attributes.clone();
                    let response = self.execute_query_or_mutation(operation).await;

                    sender.send(response).await.ok();

                    Err(Some(attributes))
                } else {
                    Ok((self, operation))
                }
            })
            .await;

        let (ctx, operation) = match result {
            Some(Ok((ctx, operation))) => (ctx, operation),
            Some(Err(attributes)) => return attributes,
            None => {
                sender.send(errors::response::gateway_timeout()).await.ok();
                return None;
            }
        };

        let attributes = operation.attributes.clone();
        ctx.execute_subscription(operation, sender).await;

        Some(attributes)
    }
}
