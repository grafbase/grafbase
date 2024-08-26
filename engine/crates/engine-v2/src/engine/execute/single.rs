use ::runtime::hooks::Hooks;
use engine_parser::types::OperationType;
use grafbase_telemetry::{
    metrics::{GraphqlErrorAttributes, GraphqlRequestMetricsAttributes, OperationMetricsAttributes},
    span::{gql::GqlRequestSpan, GqlRecorderSpanExt, GRAFBASE_TARGET},
};
use tracing::Instrument;
use web_time::Instant;

use crate::{
    engine::RequestContext,
    execution::PreExecutionContext,
    request::Request,
    response::{RequestErrorResponse, Response},
    Engine, Runtime,
};

impl<R: Runtime> Engine<R> {
    pub(super) async fn execute_single(
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

            if let Some(operation_metrics_attributes) = operation_metrics_attributes.clone() {
                span.record_gql_request((&operation_metrics_attributes).into());

                self.metrics.record_operation_duration(
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

            if let Some(attributes) = operation_metrics_attributes {
                for error_code in response.distinct_error_codes() {
                    self.metrics.increment_graphql_errors(GraphqlErrorAttributes {
                        code: error_code.into(),
                        operation_name: attributes.name.clone(),
                        client: request_context.client.clone(),
                    });
                }
            }

            response
        }
        .instrument(span.clone())
        .await
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
}
