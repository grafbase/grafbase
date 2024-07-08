use schema::Schema;

use crate::response::{ErrorCode, GraphqlError};

use super::{parse::ParsedOperation, Operation, OperationMetadata, Variables};

#[derive(Debug, thiserror::Error)]
pub enum OperationError {
    #[error(transparent)]
    Parse(#[from] super::parse::ParseError),
    #[error("{err}")]
    Bind {
        operation_metadata: Box<Option<OperationMetadata>>,
        err: super::bind::BindError,
    },
    #[error("{err}")]
    Validation {
        operation_metadata: Box<Option<OperationMetadata>>,
        err: super::validation::ValidationError,
    },
    #[error("{err}")]
    Solve {
        operation_metadata: Box<Option<OperationMetadata>>,
        err: crate::plan::PlanningError,
    },
    #[error("Failed to normalize query")]
    NormalizationError,
}

impl From<OperationError> for GraphqlError {
    fn from(err: OperationError) -> Self {
        match err {
            OperationError::Bind { err, .. } => err.into(),
            OperationError::Validation { err, .. } => err.into(),
            OperationError::Parse(err) => err.into(),
            OperationError::Solve { err, .. } => err.into(),
            OperationError::NormalizationError => GraphqlError::new(err.to_string(), ErrorCode::InternalServerError),
        }
    }
}

impl OperationError {
    pub fn take_operation_metadata(&mut self) -> Option<OperationMetadata> {
        match self {
            OperationError::Bind { operation_metadata, .. } => std::mem::take(operation_metadata),
            OperationError::Validation { operation_metadata, .. } => std::mem::take(operation_metadata),
            OperationError::Solve { operation_metadata, .. } => std::mem::take(operation_metadata),
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
    pub fn build(schema: &Schema, request: &engine::Request) -> Result<Self, OperationError> {
        let parsed_operation = super::parse::parse_operation(request)?;
        let operation_metadata = prepare_metadata(&parsed_operation, request);

        let mut operation = match super::bind::bind(schema, parsed_operation) {
            Ok(operation) => operation,
            Err(err) => {
                return Err(OperationError::Bind {
                    operation_metadata: Box::new(operation_metadata),
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
                operation_metadata: Box::new(operation_metadata),
                err,
            });
        }

        if let Err(err) = crate::plan::solve(schema, &variables, &mut operation) {
            return Err(OperationError::Solve {
                operation_metadata: Box::new(operation_metadata),
                err,
            });
        }

        operation.metadata = operation_metadata.ok_or(OperationError::NormalizationError)?;

        Ok(operation)
    }
}

fn prepare_metadata(operation: &ParsedOperation, request: &engine::Request) -> Option<OperationMetadata> {
    operation_normalizer::normalize(request.query(), request.operation_name())
        .ok()
        .map(|normalized_query| OperationMetadata {
            ty: operation.definition.ty,
            name: operation.name.clone().or_else(|| {
                engine_parser::find_first_field_name(&operation.fragments, &operation.definition.selection_set)
            }),
            normalized_query_hash: blake3::hash(normalized_query.as_bytes()).into(),
            normalized_query,
        })
}
