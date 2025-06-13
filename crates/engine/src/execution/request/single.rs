use std::{sync::Arc, time::Instant};

use grafbase_telemetry::{
    graphql::OperationType,
    metrics::{GraphqlErrorAttributes, GraphqlRequestMetricsAttributes},
    span::graphql::GraphqlOperationSpan,
};
use operation::Request;
use tracing::Instrument;

use crate::{
    Engine, Runtime,
    engine::ExtensionContext,
    prepare::PrepareContext,
    response::{ErrorCode, GraphqlError, Response},
};

use super::{RequestContext, default_response_extensions, response_extension_for_prepared_operation};

impl<R: Runtime> Engine<R> {
    pub(super) async fn execute_single(
        self: &Arc<Self>,
        request_context: &Arc<RequestContext<ExtensionContext<R>>>,
        request: Request,
    ) -> Response {
        let start = Instant::now();
        let span = GraphqlOperationSpan::default();

        async {
            let ctx = PrepareContext::new(self, request_context);
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
    async fn execute_single(mut self, request: Request) -> Response {
        let operation = match self.prepare_operation(request).await {
            Ok(operation) => operation,
            Err(response) => {
                return response.with_extensions(default_response_extensions::<R>(self.schema(), self.request_context));
            }
        };

        let attributes = operation.attributes();

        let extensions =
            response_extension_for_prepared_operation::<R>(self.schema(), self.request_context, &operation);

        if matches!(operation.cached.ty(), OperationType::Subscription) {
            let error = GraphqlError::new(
                "Subscriptions are only suported on streaming transports. Try making a request with SSE or WebSockets",
                ErrorCode::BadRequest,
            );

            return Response::request_error([error])
                .with_operation_attributes(attributes)
                .with_extensions(extensions);
        }

        self.execute_query_or_mutation(operation)
            .await
            .with_operation_attributes(attributes)
            .with_extensions(extensions)
    }
}
