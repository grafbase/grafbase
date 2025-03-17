use std::{sync::Arc, time::Instant};

use grafbase_telemetry::{
    graphql::OperationType,
    metrics::{GraphqlErrorAttributes, GraphqlRequestMetricsAttributes},
    span::graphql::GraphqlOperationSpan,
};
use operation::Request;
use runtime::hooks::Hooks;
use tracing::Instrument;

use crate::{
    Engine, Runtime,
    engine::WasmContext,
    prepare::PrepareContext,
    response::{ErrorCode, GraphqlError, Response},
};

use super::RequestContext;

impl<R: Runtime> Engine<R> {
    pub(super) async fn execute_single(
        self: &Arc<Self>,
        request_context: &Arc<RequestContext>,
        wasm_context: WasmContext<R>,
        request: Request,
    ) -> Response<<R::Hooks as Hooks>::OnOperationResponseOutput> {
        let start = Instant::now();
        let span = GraphqlOperationSpan::default();

        async {
            let ctx = PrepareContext::new(self, request_context, wasm_context);
            let response = ctx.execute_single(request).await;
            let status = response.graphql_status();
            let errors_count_by_code = response.error_code_counter().to_vec();
            span.record_response(status, &errors_count_by_code);

            if let Some(operation) = response.operation_attributes().cloned() {
                span.record_operation(&operation);

                for (error_code, _) in errors_count_by_code {
                    self.runtime.metrics().increment_graphql_errors(GraphqlErrorAttributes {
                        code: error_code.into(),
                        operation_name: operation.name.original().map(str::to_string),
                        client: request_context.client.clone(),
                    });
                }

                self.runtime.metrics().record_query_or_mutation_duration(
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

impl<R: Runtime> PrepareContext<'_, R> {
    async fn execute_single(mut self, request: Request) -> Response<<R::Hooks as Hooks>::OnOperationResponseOutput> {
        let operation = match self.prepare_operation(request).await {
            Ok(operation) => operation,
            Err(response) => return response.with_grafbase_extension(self.grafbase_response_extension(None)),
        };

        if matches!(operation.cached.ty(), OperationType::Subscription) {
            let error = GraphqlError::new(
                "Subscriptions are only suported on streaming transports. Try making a request with SSE or WebSockets",
                ErrorCode::BadRequest,
            );
            return Response::request_error([error])
                .with_operation_attributes(operation.attributes())
                .with_grafbase_extension(self.grafbase_response_extension(Some(&operation)));
        }

        let response_ext = self.grafbase_response_extension(Some(&operation));
        self.execute_query_or_mutation(operation)
            .await
            .with_grafbase_extension(response_ext)
    }
}
