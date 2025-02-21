use std::sync::Arc;

use operation::{Operation, RawVariables, Variables};
use runtime::hooks::Hooks;

use crate::{
    ErrorCode, Runtime,
    prepare::{PrepareContext, PreparedOperation},
    response::{GraphqlError, Response},
};

use super::{OperationDocument, mutation_not_allowed_with_safe_method};

impl<R: Runtime> PrepareContext<'_, R> {
    pub(super) async fn prepare_operation_without_cache(
        &mut self,
        document: OperationDocument<'_>,
        variables: RawVariables,
    ) -> Result<PreparedOperation, Response<<R::Hooks as Hooks>::OnOperationResponseOutput>> {
        if document.content.len() >= self.schema().settings.executable_document_limit_bytes {
            return Err(Response::request_error(
                None,
                [GraphqlError::new(
                    "Executable document exceeded the maximum configured size",
                    ErrorCode::OperationValidationError,
                )],
            ));
        }

        let operation = match Operation::parse(self.schema(), document.operation_name(), &document.content) {
            Ok(operation) => operation,
            Err(err) => {
                return Err(match err {
                    operation::Error::Parsing { message, locations } => Response::request_error(
                        None,
                        [GraphqlError::new(message, ErrorCode::OperationParsingError).with_locations(locations)],
                    ),
                    operation::Error::Validation {
                        message,
                        locations,
                        attributes,
                    } => Response::request_error(
                        Some(attributes.with_complexity_cost(None)),
                        [GraphqlError::new(message, ErrorCode::OperationValidationError).with_locations(locations)],
                    ),
                });
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
        if operation.attributes.ty.is_mutation() && !self.request_context.mutations_allowed {
            return Err(mutation_not_allowed_with_safe_method());
        }

        let variables = match Variables::bind(self.schema(), &operation, variables) {
            Ok(variables) => variables,
            Err(errors) => {
                return Err(Response::request_error(
                    Some(operation.attributes.with_complexity_cost(None)),
                    errors,
                ));
            }
        };

        let complexity_cost = match operation.compute_and_validate_complexity(self.schema(), &variables) {
            Ok(cost) => cost,
            Err(err) => {
                return Err(Response::request_error(
                    Some(operation.attributes.with_complexity_cost(None)),
                    [GraphqlError::new(err.to_string(), ErrorCode::OperationValidationError)],
                ));
            }
        };

        let attributes = operation.attributes.clone();
        let cached = match crate::prepare::solve(self.schema(), document, operation) {
            Ok(plan) => plan,
            Err(err) => {
                return Err(Response::request_error(
                    Some(attributes.with_complexity_cost(complexity_cost)),
                    [err],
                ));
            }
        };

        let plan = match crate::prepare::plan(self, &cached, &variables).await {
            Ok(plan) => plan,
            Err(err) => {
                return Err(Response::request_error(
                    Some(attributes.with_complexity_cost(complexity_cost)),
                    [err],
                ));
            }
        };

        Ok(PreparedOperation {
            cached: Arc::new(cached),
            plan,
            variables,
            complexity_cost,
        })
    }
}
