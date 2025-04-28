use std::sync::Arc;

use operation::{Operation, RawVariables, Variables};
use runtime::hooks::Hooks;

use crate::{
    ErrorCode, Runtime,
    mcp::McpResponseExtension,
    prepare::{PrepareContext, PreparedOperation},
    response::{GraphqlError, Response, ResponseExtensions},
};

use super::{OperationDocument, mutation_not_allowed_with_safe_method};

impl<R: Runtime> PrepareContext<'_, R> {
    pub(super) async fn prepare_operation_without_cache(
        &mut self,
        document: OperationDocument<'_>,
        variables: RawVariables,
    ) -> Result<PreparedOperation, Response<<R::Hooks as Hooks>::OnOperationResponseOutput>> {
        if document.content.len() >= self.schema().settings.executable_document_limit_bytes {
            let error = GraphqlError::new(
                "Executable document exceeded the maximum configured size",
                ErrorCode::OperationValidationError,
            );
            return Err(Response::request_error([error]));
        }

        let operation = match Operation::parse(self.schema(), document.operation_name(), &document.content) {
            Ok(operation) => operation,
            Err(operation::Errors { items, attributes }) => {
                let resp = if self.request_context.include_mcp_response_extension {
                    let site_ids = items.iter().filter_map(|error| error.site_id).collect::<Vec<_>>();
                    Response::request_error(items.into_iter().map(operation_error_into_graphql_error)).with_extensions(
                        ResponseExtensions {
                            mcp: Some(McpResponseExtension { site_ids }),
                            ..Default::default()
                        },
                    )
                } else {
                    Response::request_error(items.into_iter().map(operation_error_into_graphql_error))
                };

                return Err(if let Some(attributes) = attributes {
                    resp.with_operation_attributes(attributes.with_complexity_cost(None))
                } else {
                    resp
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
        if operation.attributes.ty.is_mutation() && !self.request_context.can_mutate {
            return Err(mutation_not_allowed_with_safe_method());
        }

        let variables = match Variables::bind(self.schema(), &operation, variables) {
            Ok(variables) => variables,
            Err(errors) => {
                return Err(Response::request_error(errors.into_iter().map(|err| {
                    GraphqlError::new(err.message, ErrorCode::VariableError).with_locations(err.locations)
                }))
                .with_operation_attributes(operation.attributes.clone().with_complexity_cost(None)));
            }
        };

        let complexity_cost = match operation.compute_and_validate_complexity(self.schema(), &variables) {
            Ok(cost) => cost,
            Err(err) => {
                let error = GraphqlError::new(err.to_string(), ErrorCode::OperationValidationError);
                return Err(Response::request_error([error])
                    .with_operation_attributes(operation.attributes.clone().with_complexity_cost(None)));
            }
        };

        let attributes = operation.attributes.clone();
        let cached = match crate::prepare::solve(self.schema(), document, operation) {
            Ok(plan) => plan,
            Err(err) => {
                return Err(Response::request_error([err])
                    .with_operation_attributes(attributes.with_complexity_cost(complexity_cost)));
            }
        };

        let plan = match crate::prepare::plan(self, &cached, &variables).await {
            Ok(plan) => plan,
            Err(response) => {
                return Err(response.with_operation_attributes(
                    cached
                        .operation
                        .attributes
                        .clone()
                        .with_complexity_cost(complexity_cost),
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

fn operation_error_into_graphql_error(err: operation::Error) -> GraphqlError {
    let code = match err.kind {
        operation::ErrorKind::Parsing => ErrorCode::OperationParsingError,
        operation::ErrorKind::Validation => ErrorCode::OperationValidationError,
    };
    GraphqlError::new(err.message, code).with_locations(err.locations)
}
