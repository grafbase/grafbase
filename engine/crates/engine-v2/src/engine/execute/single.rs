use ::runtime::hooks::Hooks;
use engine_parser::types::OperationType;
use grafbase_telemetry::{
    metrics::{GraphqlErrorAttributes, GraphqlRequestMetricsAttributes, OperationMetricsAttributes},
    span::{gql::GqlRequestSpan, GqlRecorderSpanExt},
};
use runtime::hooks;
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
    ) -> (Response, Option<Vec<u8>>) {
        let start = Instant::now();
        let span = GqlRequestSpan::create();

        async {
            let ctx = PreExecutionContext::new(self, request_context);

            let (operation_metrics_attributes, on_operation_response_output, response) =
                ctx.execute_single(request).await;

            let elapsed = start.elapsed();
            let status = response.graphql_status();

            if let Some(operation_metrics_attributes) = operation_metrics_attributes.clone() {
                span.record_gql_request((&operation_metrics_attributes).into());

                self.runtime.metrics().record_operation_duration(
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

            if let Some(attributes) = operation_metrics_attributes {
                for error_code in response.distinct_error_codes() {
                    self.runtime.metrics().increment_graphql_errors(GraphqlErrorAttributes {
                        code: error_code.into(),
                        operation_name: attributes.name.clone(),
                        client: request_context.client.clone(),
                    });
                }
            }

            // After recording all operation metadata
            tracing::debug!("Executed operation");

            (response, on_operation_response_output)
        }
        .instrument(span.clone())
        .await
    }
}

type OperationResponseHookResult = Vec<u8>;

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    async fn execute_single(
        mut self,
        request: Request,
    ) -> (
        Option<OperationMetricsAttributes>,
        Option<OperationResponseHookResult>,
        Response,
    ) {
        let mut operation_info = hooks::ExecutedOperation::builder();

        let operation_plan = match self.prepare_operation(request, &mut operation_info).await {
            Ok(operation_plan) => operation_plan,
            Err((metadata, response)) => return (metadata, None, response),
        };

        operation_info.track_prepare();

        let metrics_attributes = operation_plan.metrics_attributes.clone();

        if matches!(operation_plan.ty(), OperationType::Subscription) {
            let response = RequestErrorResponse::bad_request_but_well_formed_graphql_over_http_request(
                "Subscriptions are only suported on streaming transports. Try making a request with SSE or WebSockets",
            );

            (Some(metrics_attributes), None, response)
        } else {
            operation_info.set_name(metrics_attributes.name.as_ref());

            let hooks = self.hooks();
            let mut response = self.execute_query_or_mutation(operation_plan).await;

            operation_info.set_on_subgraph_response_outputs(response.take_on_subgraph_response_outputs());

            let executed_operation =
                operation_info.finalize(&metrics_attributes.sanitized_query, response.graphql_status());

            match hooks.on_operation_response(executed_operation).await {
                Ok(operation_result) => (
                    Some(metrics_attributes),
                    Some(operation_result),
                    Response::Executed(response),
                ),
                Err(e) => (Some(metrics_attributes), None, Response::execution_error([e])),
            }
        }
    }
}
