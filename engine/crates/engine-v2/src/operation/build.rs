use grafbase_tracing::{gql_response_status::GraphqlResponseStatus, metrics::GraphqlOperationMetricsAttributes};
use schema::Schema;

use crate::{engine::RequestContext, response::GraphqlError};

use super::{Operation, Variables};

#[derive(Debug, thiserror::Error)]
pub enum OperationError {
    #[error(transparent)]
    Parse(#[from] super::parse::ParseError),
    #[error("{err}")]
    Bind {
        operation_attributes: Box<Option<GraphqlOperationMetricsAttributes>>,
        err: super::bind::BindError,
    },
    #[error("{err}")]
    Validation {
        operation_attributes: Box<Option<GraphqlOperationMetricsAttributes>>,
        err: super::validation::ValidationError,
    },
    #[error("{err}")]
    Solve {
        operation_attributes: Box<Option<GraphqlOperationMetricsAttributes>>,
        err: crate::plan::PlanningError,
    },
}

impl From<OperationError> for GraphqlError {
    fn from(err: OperationError) -> Self {
        match err {
            OperationError::Bind { err, .. } => err.into(),
            OperationError::Validation { err, .. } => err.into(),
            OperationError::Parse(err) => err.into(),
            OperationError::Solve { err, .. } => err.into(),
        }
    }
}

impl OperationError {
    pub fn take_operation_attributes(&mut self) -> Option<GraphqlOperationMetricsAttributes> {
        match self {
            OperationError::Bind {
                operation_attributes, ..
            } => std::mem::take(operation_attributes),
            OperationError::Validation {
                operation_attributes, ..
            } => std::mem::take(operation_attributes),
            OperationError::Solve {
                operation_attributes, ..
            } => std::mem::take(operation_attributes),
            _ => None,
        }
    }
}

impl Operation {
    /// Builds an `Operation` by binding unbound operation to a schema and configuring its non functional requirements
    /// like caching, auth, ....
    ///
    /// All field names are mapped to their actual field id in the schema and respective configuration.
    /// At this stage the operation might not be resolvable but it should make sense given the schema types.
    pub fn build<C>(
        schema: &Schema,
        // FIXME: build shouldn't depend on it.
        request_context: &RequestContext<C>,
        request: &engine::Request,
    ) -> Result<(Self, Option<GraphqlOperationMetricsAttributes>), OperationError> {
        let parsed_operation = super::parse::parse_operation(request)?;
        let operation_attributes = operation_normalizer::normalize(request.query(), request.operation_name())
            .ok()
            .map(|normalized_query| GraphqlOperationMetricsAttributes {
                normalized_query_hash: blake3::hash(normalized_query.as_bytes()).into(),
                name: parsed_operation.name.clone().or_else(|| {
                    engine_parser::find_first_field_name(
                        &parsed_operation.fragments,
                        &parsed_operation.definition.selection_set,
                    )
                }),
                ty: parsed_operation.definition.ty.as_str(),
                normalized_query,
                // overridden at the end.
                status: GraphqlResponseStatus::Success,
                cache_status: None,
                client: request_context.client.clone(),
            });

        let mut operation = match super::bind::bind(schema, parsed_operation) {
            Ok(operation) => operation,
            Err(err) => {
                return Err(OperationError::Bind {
                    operation_attributes: Box::new(operation_attributes),
                    err,
                })
            }
        };

        // At this stage we don't take into account variables so we can cache the result.
        let variables = Variables::create_unavailable_for(&operation);
        if let Err(err) =
            super::validation::validate_operation(schema, operation.walker_with(schema.walker(), &variables), request)
        {
            return Err(OperationError::Validation {
                operation_attributes: Box::new(operation_attributes),
                err,
            });
        }

        if let Err(err) = crate::plan::solve(schema, &variables, &mut operation) {
            return Err(OperationError::Solve {
                operation_attributes: Box::new(operation_attributes),
                err,
            });
        }

        Ok((operation, operation_attributes))
    }
}
