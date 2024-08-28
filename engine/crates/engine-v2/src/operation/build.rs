use schema::Schema;

use crate::{
    request::Request,
    response::{ErrorCode, GraphqlError},
};

use super::{
    bind::{bind_operation, BindError},
    blueprint::ResponseBlueprintBuilder,
    logical_planner::{LogicalPlanner, LogicalPlanningError},
    metrics::{generate_used_fields, prepare_metrics_attributes},
    parse::{parse_operation, ParseError},
    validation::{validate_operation, ValidationError},
    Operation, OperationMetricsAttributes, PreparedOperation,
};

#[derive(Debug, thiserror::Error)]
pub(crate) enum OperationError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error("{err}")]
    Bind {
        metrics_attributes: Box<Option<OperationMetricsAttributes>>,
        err: BindError,
    },
    #[error("{err}")]
    Validation {
        metrics_attributes: Box<Option<OperationMetricsAttributes>>,
        err: ValidationError,
    },
    #[error("{err}")]
    LogicalPlanning {
        metrics_attributes: Box<Option<OperationMetricsAttributes>>,
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
    pub fn take_metrics_attributes(&mut self) -> Option<OperationMetricsAttributes> {
        match self {
            OperationError::Bind { metrics_attributes, .. } => std::mem::take(metrics_attributes),
            OperationError::Validation { metrics_attributes, .. } => std::mem::take(metrics_attributes),
            OperationError::LogicalPlanning { metrics_attributes, .. } => std::mem::take(metrics_attributes),
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
    pub fn build(schema: &Schema, request: &Request, document: &str) -> Result<PreparedOperation, OperationError> {
        let parsed_operation = parse_operation(request.operation_name.as_deref(), document)?;
        let metrics_attributes = prepare_metrics_attributes(&parsed_operation, document);

        let mut operation = match bind_operation(schema, parsed_operation) {
            Ok(operation) => operation,
            Err(err) => {
                return Err(OperationError::Bind {
                    metrics_attributes: Box::new(metrics_attributes),
                    err,
                })
            }
        };

        if let Err(err) = validate_operation(schema, operation.walker_with(schema.walker())) {
            return Err(OperationError::Validation {
                metrics_attributes: Box::new(metrics_attributes),
                err,
            });
        }

        let plan = match LogicalPlanner::new(schema, &mut operation).plan() {
            Ok(plan) => plan,
            Err(err) => {
                return Err(OperationError::LogicalPlanning {
                    metrics_attributes: Box::new(metrics_attributes),
                    err,
                });
            }
        };

        let response_blueprint = ResponseBlueprintBuilder::new(schema, &operation, &plan).build();

        let mut metrics_attributes = metrics_attributes.ok_or(OperationError::NormalizationError)?;
        metrics_attributes.used_fields = generate_used_fields(schema, &operation);

        Ok(PreparedOperation {
            operation,
            metrics_attributes,
            plan,
            response_blueprint,
        })
    }
}
