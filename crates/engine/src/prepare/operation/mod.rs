mod after_variables;
mod before_variables;
mod complexity_control;

use ::runtime::operation_cache::OperationCache;
use config::ComplexityControl;
use futures::FutureExt;
use runtime::hooks::Hooks;
use schema::Settings;
use std::sync::Arc;

use crate::{
    operation::Variables,
    request::Request,
    response::{GraphqlError, Response},
    ErrorCode, Runtime,
};

use super::{error::PrepareResult, trusted_documents::OperationDocument, PrepareContext, PreparedOperation};

impl<'ctx, R: Runtime> PrepareContext<'ctx, R> {
    pub(super) async fn prepare_operation_inner(
        &mut self,
        request: Request,
    ) -> Result<PreparedOperation, Response<<R::Hooks as Hooks>::OnOperationResponseOutput>> {
        let result = {
            let OperationDocument { cache_key, load_fut } = match self.determine_operation_document(&request) {
                Ok(doc) => doc,
                // If we have an error a this stage, it means we couldn't determine what document
                // to load, so we don't consider it a well-formed GraphQL-over-HTTP request.
                Err(err) => return Err(Response::refuse_request_with(http::StatusCode::BAD_REQUEST, vec![err])),
            };

            if let Some(operation) = self.operation_cache().get(&cache_key).await {
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

        let cached_operation = match result {
            Ok(op) => op,
            Err((cache_key, document)) => {
                let cached_operation =
                    self.build_cached_operation(&request, &document)
                        .map(Arc::new)
                        .map_err(|mut err| {
                            let attributes = err.take_operation_attributes();
                            Response::request_error(attributes, [err])
                        })?;

                let cache_fut = self.operation_cache().insert(cache_key, cached_operation.clone());
                self.push_background_future(cache_fut.boxed());

                cached_operation
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
        if cached_operation.ty().is_mutation() && !self.request_context.mutations_allowed {
            return Err(mutation_not_allowed_with_safe_method());
        }

        let variables = Variables::build(self.schema(), &cached_operation, request.variables).map_err(|errors| {
            Response::request_error(Some(cached_operation.operation_attributes_for_error()), errors)
        })?;

        let prepared_operation = self
            .prepare_cached_operation(Arc::clone(&cached_operation), variables)
            .await
            .map_err(|err| Response::request_error(Some(cached_operation.operation_attributes_for_error()), [err]))?;

        validate_prepared_operation(&prepared_operation, &self.schema().settings)
            .map_err(|err| Response::request_error(Some(prepared_operation.attributes()), [err]))?;

        Ok(prepared_operation)
    }
}

fn mutation_not_allowed_with_safe_method<OnOperationResponseHookOutput>() -> Response<OnOperationResponseHookOutput> {
    Response::refuse_request_with(
        http::StatusCode::METHOD_NOT_ALLOWED,
        vec![GraphqlError::new(
            "Mutation is not allowed with a safe method like GET",
            ErrorCode::BadRequest,
        )],
    )
}

fn validate_prepared_operation(operation: &PreparedOperation, settings: &Settings) -> PrepareResult<()> {
    let ComplexityControl::Enforce { limit, .. } = settings.complexity_control else {
        return Ok(());
    };
    let Some(complexity) = operation.complexity else {
        tracing::error!("Complexity control is enabled but complexity could not be calculated!");
        return Ok(());
    };
    if complexity > limit {
        return Err(super::error::PrepareError::ComplexityLimitReached);
    }

    Ok(())
}
