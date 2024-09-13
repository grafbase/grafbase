use engine_parser::types::OperationType;
use grafbase_telemetry::{
    metrics::{GraphqlErrorAttributes, GraphqlRequestMetricsAttributes},
    span::graphql::GraphqlOperationSpan,
};
use tracing::Instrument;
use web_time::Instant;

use crate::{
    engine::{HooksContext, RequestContext},
    execution::PreExecutionContext,
    request::Request,
    response::{GraphqlError, Response},
    Engine, ErrorCode, Runtime,
};

impl<R: Runtime> Engine<R> {
    pub(super) async fn execute_single(
        &self,
        request_context: &RequestContext,
        hooks_context: HooksContext<R>,
        request: Request,
    ) -> Response {
        let start = Instant::now();
        let span = GraphqlOperationSpan::default();

        async {
            let ctx = PreExecutionContext::new(self, request_context, hooks_context);

            let response = ctx.execute_single(request).await;

            let status = response.graphql_status();
            span.record_response_status(status);
            span.record_distinct_error_codes(response.distinct_error_codes());

            if let Some(operation) = response.operation_attributes().cloned() {
                span.record_operation(&operation);

                for error_code in response.distinct_error_codes() {
                    self.runtime.metrics().increment_graphql_errors(GraphqlErrorAttributes {
                        code: error_code.into(),
                        operation_name: operation.name.original().map(str::to_string),
                        client: request_context.client.clone(),
                    });
                }
                self.runtime.metrics().record_operation_duration(
                    GraphqlRequestMetricsAttributes {
                        operation,
                        status,
                        cache_status: None,
                        client: request_context.client.clone(),
                    },
                    start.elapsed(),
                );
            }

            // After recording all operation metadata
            tracing::debug!("Executed operation");

            response
        }
        .instrument(span.clone())
        .await
    }
}

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    async fn execute_single(mut self, request: Request) -> Response {
        let operation = match self.prepare_operation(request).await {
            Ok(operation) => operation,
            Err(response) => return response,
        };

        if matches!(operation.ty(), OperationType::Subscription) {
            let response = Response::request_error(
                Some(operation.attributes.clone()),
                [GraphqlError::new("Subscriptions are only suported on streaming transports. Try making a request with SSE or WebSockets", ErrorCode::BadRequest)],
            );
            return response;
        }

        self.execute_query_or_mutation(operation).await
    }
}
