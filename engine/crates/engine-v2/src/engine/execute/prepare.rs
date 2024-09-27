use ::runtime::operation_cache::OperationCache;
use futures::FutureExt;
use std::sync::Arc;

use crate::{
    engine::trusted_documents::OperationDocument,
    execution::{ExecutableOperation, PreExecutionContext},
    operation::{Operation, Variables},
    request::Request,
    response::Response,
    Runtime,
};

impl<'ctx, R: Runtime> PreExecutionContext<'ctx, R> {
    #[tracing::instrument(skip_all)]
    pub(super) async fn prepare_operation(&mut self, request: Request) -> Result<ExecutableOperation, Response> {
        let result = self.prepare_operation_inner(request).await;
        let duration = self.executed_operation_builder.track_prepare();

        match result {
            Ok(operation) => {
                self.metrics()
                    .record_successful_preparation_duration(operation.attributes.clone(), duration);

                Ok(operation)
            }
            Err(response) => {
                self.metrics()
                    .record_failed_preparation_duration(response.operation_attributes().cloned(), duration);

                Err(response)
            }
        }
    }

    async fn prepare_operation_inner(&mut self, request: Request) -> Result<ExecutableOperation, Response> {
        let result = {
            let OperationDocument { cache_key, load_fut } = match self.determine_operation_document(&request) {
                Ok(doc) => doc,
                // If we have an error a this stage, it means we couldn't determine what document
                // to load, so we don't consider it a well-formed GraphQL-over-HTTP request.
                Err(err) => return Err(Response::refuse_request_with(http::StatusCode::BAD_REQUEST, err)),
            };

            if let Some(operation) = self.engine.operation_cache.get(&cache_key).await {
                self.executed_operation_builder.set_cached_plan();
                self.metrics().record_operation_cache_hit();

                Ok(operation)
            } else {
                self.metrics().record_operation_cache_miss();
                match load_fut.await {
                    Ok(document) => Err((cache_key, document)),
                    Err(err) => return Err(Response::request_error(None, [err])),
                }
            }
        };

        let operation = match result {
            Ok(operation) => operation,
            Err((cache_key, document)) => {
                let operation = Operation::prepare(self.schema(), &request, &document)
                    .map(Arc::new)
                    .map_err(|mut err| {
                        let attributes = err.take_operation_attributes();
                        Response::request_error(attributes, [err])
                    })?;

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
            return Err(Response::method_not_allowed(
                "Mutation is not allowed with a safe method like GET",
            ));
        }

        let variables = Variables::build(self.schema(), &operation, request.variables)
            .map_err(|errors| Response::request_error(Some(operation.attributes.clone()), errors))?;

        self.finalize_operation(Arc::clone(&operation), variables)
            .await
            .map_err(|err| Response::request_error(Some(operation.attributes.clone()), [err]))
    }
}
