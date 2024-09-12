use std::sync::Arc;

use ::runtime::hooks::Hooks;
use async_runtime::stream::StreamExt;
use engine_parser::types::OperationType;
use futures::channel::mpsc;
use futures_util::{SinkExt, Stream};
use grafbase_telemetry::{
    grafbase_client::Client,
    graphql::{GraphqlOperationAttributes, GraphqlResponseStatus},
    metrics::{EngineMetrics, GraphqlErrorAttributes, GraphqlRequestMetricsAttributes},
    span::graphql::GraphqlOperationSpan,
};
use runtime::hooks;
use tracing::Instrument;
use web_time::Instant;

use crate::{
    engine::{RequestContext, RuntimeExt},
    execution::PreExecutionContext,
    request::Request,
    response::{RequestErrorResponse, Response},
    Engine, Runtime,
};

impl<R: Runtime> Engine<R> {
    pub(super) fn execute_stream(
        self: &Arc<Self>,
        request_context: Arc<RequestContext<<R::Hooks as Hooks>::Context>>,
        request: Request,
    ) -> impl Stream<Item = Response> + Send + 'static {
        let start = Instant::now();
        let engine = Arc::clone(self);
        let (sender, receiver) = mpsc::channel(2);

        let span = GraphqlOperationSpan::default();
        let span_clone = span.clone();

        receiver.join(
            async move {
                let ctx = PreExecutionContext::new(&engine, &request_context);
                let (operation_attributes, status) = ctx.execute_stream(request, sender).await;
                let elapsed = start.elapsed();

                if let Some(attributes) = operation_attributes {
                    span.record_operation(&attributes);

                    engine.runtime.metrics().record_operation_duration(
                        GraphqlRequestMetricsAttributes {
                            operation: attributes,
                            status,
                            cache_status: None,
                            client: request_context.client.clone(),
                        },
                        elapsed,
                    );
                }

                span.record_response_status(status);
                // After recording all operation metadata
                tracing::debug!("Executed operation in stream.")
            }
            .instrument(span_clone),
        )
    }
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    async fn execute_stream(
        mut self,
        request: Request,
        mut sender: mpsc::Sender<Response>,
    ) -> (Option<GraphqlOperationAttributes>, GraphqlResponseStatus) {
        let engine = self.engine;
        let client = self.request_context.client.clone();

        // If it's a subscription, we at least have a timeout on the operation preparation.
        let result = engine
            .runtime
            .with_timeout(self.engine.schema.settings.timeout, async {
                // TODO: we should figure out how the access logs look like for subscriptions in another PR.
                let mut operation_info = hooks::ExecutedOperation::builder();

                let operation = match self.prepare_operation(request, &mut operation_info).await {
                    Ok(operation) => operation,
                    Err((metadata, response)) => {
                        let status = response.graphql_status();
                        sender.send(response).await.ok();
                        return Err((metadata, status));
                    }
                };

                if matches!(operation.ty(), OperationType::Query | OperationType::Mutation) {
                    let metrics_attributes = Some(operation.attributes.clone());
                    let response = self.execute_query_or_mutation(operation).await;
                    let status = response.graphql_status();

                    sender.send(Response::Executed(response)).await.ok();

                    Err((metrics_attributes, status))
                } else {
                    Ok((self, operation))
                }
            })
            .await;

        let (ctx, operation) = match result {
            Some(Ok((ctx, operation))) => (ctx, operation),
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
            operation_name: Option<String>,
            client: Option<Client>,
            metrics: &'a EngineMetrics,
        }

        impl crate::execution::ResponseSender for Sender<'_> {
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

                self.sender.send(response).await
            }
        }

        let metrics_attributes = Some(operation.attributes.clone());

        ctx.execute_subscription(
            operation,
            Sender {
                sender,
                status: &mut status,
                operation_name: metrics_attributes
                    .as_ref()
                    .and_then(|a| a.name.original().map(str::to_string)),
                client: client.clone(),
                metrics: engine.runtime.metrics(),
            },
        )
        .await;

        (metrics_attributes, status)
    }
}
