use ::runtime::operation_cache::OperationCache;
use futures::FutureExt;
use grafbase_telemetry::metrics::{OperationMetricsAttributes, QueryPreparationAttributes};
use runtime::hooks::ExecutedOperationBuilder;
use std::sync::Arc;
use web_time::Instant;

use crate::{
    engine::trusted_documents::OperationDocument,
    execution::{ExecutableOperation, PreExecutionContext},
    operation::{Operation, Variables},
    request::Request,
    response::{RefusedRequestResponse, Response},
    Runtime,
};

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    pub(super) async fn prepare_operation(
        &mut self,
        request: Request,
        operation_info: &mut ExecutedOperationBuilder,
    ) -> Result<ExecutableOperation, (Option<OperationMetricsAttributes>, Response)> {
        let start = Instant::now();
        let result = self.prepare_operation_inner(request, operation_info).await;
        let duration = start.elapsed();

        match result {
            Ok(operation) => {
                let attributes = QueryPreparationAttributes {
                    operation_name: operation.prepared.metrics_attributes.name.clone(),
                    document: Some(operation.prepared.metrics_attributes.sanitized_query.clone()),
                    success: true,
                };

                self.metrics().record_preparation_latency(attributes, duration);

                Ok(operation)
            }
            Err(e) => {
                let attributes = QueryPreparationAttributes {
                    operation_name: None,
                    document: None,
                    success: false,
                };

                self.metrics().record_preparation_latency(attributes, duration);

                Err(e)
            }
        }
    }

    async fn prepare_operation_inner(
        &mut self,
        request: Request,
        operation_info: &mut ExecutedOperationBuilder,
    ) -> Result<ExecutableOperation, (Option<OperationMetricsAttributes>, Response)> {
        let result = {
            let OperationDocument { cache_key, load_fut } = match self.determine_operation_document(&request) {
                Ok(doc) => doc,
                // If we have an error a this stage, it means we couldn't determine what document
                // to load, so we don't consider it a well-formed GraphQL-over-HTTP request.
                Err(err) => return Err((None, Response::refuse_request_with(http::StatusCode::BAD_REQUEST, err))),
            };

            if let Some(operation) = self.engine.operation_cache.get(&cache_key).await {
                operation_info.set_cached();
                self.metrics().record_operation_cache_hit();

                Ok(operation)
            } else {
                self.metrics().record_operation_cache_miss();
                match load_fut.await {
                    Ok(document) => Err((cache_key, document)),
                    Err(err) => return Err((None, Response::request_error([err]))),
                }
            }
        };

        let operation = match result {
            Ok(operation) => operation,
            Err((cache_key, document)) => {
                let operation = Operation::prepare(self.schema(), &request, &document)
                    .map(Arc::new)
                    .map_err(|mut err| (err.take_metrics_attributes(), Response::request_error([err])))?;

                let cache_fut = self.engine.operation_cache.insert(cache_key, operation.clone());
                self.push_background_future(cache_fut.boxed());

                operation
            }
        };

        // GraphQL-over-HTTP spec:
        //   GET requests MUST NOT be used for executing mutation operations. If the values of {query} and {operationName} indicate that
        //   a mutation operation is to be executed, the server MUST respond with error status code 405 (Method Not Allowed) and halt
        //   execution. This restriction is necessary to conform with the long-established semantics of safe methods within HTTP.
        //
        // While it's technically a RequestError at this stage, as we have a well-formed GraphQL-over-HTTP request,
        // we must return a 4xx without regard to the Accept header in all cases, so it's akin to denying the request.
        //
        // This error would be confusing for a websocket connection, but today mutation are always
        // allowed for it.
        if operation.ty.is_mutation() && !self.request_context.mutations_allowed {
            return Err((
                Some(operation.metrics_attributes.clone()),
                RefusedRequestResponse::method_not_allowed("Mutation is not allowed with a safe method like GET"),
            ));
        }

        let variables = Variables::build(self.schema(), &operation, request.variables).map_err(|errors| {
            (
                Some(operation.metrics_attributes.clone()),
                Response::request_error(errors),
            )
        })?;

        self.finalize_operation(Arc::clone(&operation), variables)
            .await
            .map_err(|err| {
                (
                    Some(operation.metrics_attributes.clone()),
                    Response::request_error([err]),
                )
            })
    }
}
