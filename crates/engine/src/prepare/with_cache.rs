use std::sync::Arc;

use operation::{RawVariables, Variables};
use walker::Walk;

use crate::{
    ErrorCode, Runtime,
    prepare::{CachedOperation, PrepareContext, PreparedOperation},
    response::{GraphqlError, Response},
};

use super::mutation_not_allowed_with_safe_method;

impl<R: Runtime> PrepareContext<'_, R> {
    pub(super) async fn prepare_operation_with_cache(
        &mut self,
        cached: Arc<CachedOperation>,
        variables: RawVariables,
    ) -> Result<PreparedOperation, Response> {
        // Set the operation type on the builder from the cached operation
        self.executed_operation_builder
            .operation_type(cached.operation.attributes.ty);

        // Check if any fields in the operation are deprecated
        let has_deprecated_fields = cached
            .operation
            .data_fields
            .iter()
            .any(|field| field.definition_id.walk(self.schema()).has_deprecated().is_some());

        self.executed_operation_builder
            .has_deprecated_fields(has_deprecated_fields);

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
        if cached.operation.attributes.ty.is_mutation() && !self.request_context.can_mutate {
            return Err(mutation_not_allowed_with_safe_method(
                self.schema().config.error_code_mapping.clone(),
            ));
        }

        let variables = match Variables::bind(self.schema(), &cached.operation, variables) {
            Ok(variables) => variables,
            Err(errors) => {
                return Err(Response::request_error(
                    self.schema().config.error_code_mapping.clone(),
                    errors.into_iter().map(|err| {
                        GraphqlError::new(err.message, ErrorCode::VariableError).with_locations(err.locations)
                    }),
                )
                .with_operation_attributes(cached.operation.attributes.clone().with_complexity_cost(None)));
            }
        };

        let complexity_cost = match cached
            .operation
            .compute_and_validate_complexity(self.schema(), &variables)
        {
            Ok(cost) => cost,
            Err(err) => {
                let error = GraphqlError::new(err.to_string(), ErrorCode::OperationValidationError);
                return Err(
                    Response::request_error(self.schema().config.error_code_mapping.clone(), [error])
                        .with_operation_attributes(cached.operation.attributes.clone().with_complexity_cost(None)),
                );
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
            cached,
            plan,
            variables,
            complexity_cost,
        })
    }
}
