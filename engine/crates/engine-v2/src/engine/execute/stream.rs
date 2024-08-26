use std::sync::Arc;

use ::runtime::hooks::Hooks;
use async_runtime::stream::StreamExt;
use engine_parser::types::OperationType;
use futures::channel::mpsc;
use futures_util::{SinkExt, Stream};
use grafbase_telemetry::{
    gql_response_status::GraphqlResponseStatus,
    grafbase_client::Client,
    metrics::{EngineMetrics, GraphqlErrorAttributes, GraphqlRequestMetricsAttributes, OperationMetricsAttributes},
    span::{gql::GqlRequestSpan, GqlRecorderSpanExt, GRAFBASE_TARGET},
};
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

        let span = GqlRequestSpan::create();
        let span_clone = span.clone();

        receiver.join(
            async move {
                let ctx = PreExecutionContext::new(&engine, &request_context);
                let (operation_metrics_attributes, status) = ctx.execute_stream(request, sender).await;
                let elapsed = start.elapsed();

                if let Some(operation_metrics_attributes) = operation_metrics_attributes {
                    tracing::Span::current().record_gql_request((&operation_metrics_attributes).into());

                    engine.metrics.record_operation_duration(
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
    async fn execute_stream(
        mut self,
        request: Request,
        mut sender: mpsc::Sender<Response>,
    ) -> (Option<OperationMetricsAttributes>, GraphqlResponseStatus) {
        let engine = self.engine;
        let client = self.request_context.client.clone();

        // If it's a subscription, we at least have a timeout on the operation preparation.
        let result = engine
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

        let metrics_attributes = Some(operation_plan.metrics_attributes.clone());

        ctx.execute_subscription(
            operation_plan,
            Sender {
                sender,
                status: &mut status,
                operation_name: metrics_attributes.as_ref().and_then(|a| a.name.clone()),
                client: client.clone(),
                metrics: &engine.metrics,
            },
        )
        .await;

        (metrics_attributes, status)
    }
}
