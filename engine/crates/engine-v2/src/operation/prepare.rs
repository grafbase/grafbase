use schema::Schema;

use crate::{
    request::Request,
    response::{ErrorCode, GraphqlError},
};

use super::{
    bind::{bind_operation, BindError},
    blueprint::ResponseBlueprintBuilder,
    cache_scopes::calculate_cache_scopes,
    logical_planner::{LogicalPlanner, LogicalPlanningError},
    metrics::extract_attributes,
    parse::{parse_operation, ParseError},
    validation::{validate_operation, ValidationError},
    GraphqlOperationAttributes, Operation, PreparedOperation,
};

#[derive(Debug, thiserror::Error)]
pub(crate) enum OperationError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error("{err}")]
    Bind {
        attributes: Box<Option<GraphqlOperationAttributes>>,
        err: BindError,
    },
    #[error("{err}")]
    Validation {
        attributes: Box<Option<GraphqlOperationAttributes>>,
        err: ValidationError,
    },
    #[error("{err}")]
    LogicalPlanning {
        attributes: Box<Option<GraphqlOperationAttributes>>,
        err: LogicalPlanningError,
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
            OperationError::LogicalPlanning { err, .. } => err.into(),
            OperationError::NormalizationError => GraphqlError::new(err.to_string(), ErrorCode::InternalServerError),
        }
    }
}

impl OperationError {
    pub fn take_operation_attributes(&mut self) -> Option<GraphqlOperationAttributes> {
        match self {
            OperationError::Bind { attributes, .. } => std::mem::take(attributes),
            OperationError::Validation { attributes, .. } => std::mem::take(attributes),
            OperationError::LogicalPlanning { attributes, .. } => std::mem::take(attributes),
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
    pub fn prepare(schema: &Schema, request: &Request, document: &str) -> Result<PreparedOperation, OperationError> {
        let parsed_operation = parse_operation(request.operation_name.as_deref(), document)?;
        let attributes = extract_attributes(&parsed_operation, document);

        let mut operation = match bind_operation(schema, parsed_operation) {
            Ok(operation) => operation,
            Err(err) => {
                return Err(OperationError::Bind {
                    attributes: Box::new(attributes),
                    err,
                })
            }
        };

        if let Err(err) = validate_operation(schema, operation.walker_with(schema)) {
            return Err(OperationError::Validation {
                attributes: Box::new(attributes),
                err,
            });
        }

        let plan = match LogicalPlanner::new(schema, &mut operation).plan() {
            Ok(plan) => plan,
            Err(err) => {
                return Err(OperationError::LogicalPlanning {
                    attributes: Box::new(attributes),
                    err,
                });
            }
        };

        let (logical_plan_cache_scopes, cache_scopes) = calculate_cache_scopes(operation.walker_with(schema), &plan);

        let response_blueprint = ResponseBlueprintBuilder::new(schema, &operation, &plan).build();

        let attributes = attributes.ok_or(OperationError::NormalizationError)?;

        Ok(PreparedOperation {
            operation,
            attributes,
            plan,
            response_blueprint,
            logical_plan_cache_scopes,
            cache_scopes,
        })
    }
}
