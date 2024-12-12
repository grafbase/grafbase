use grafbase_telemetry::graphql::GraphqlOperationAttributes;
use operation::{ComplexityError, OperationAttributes};

use crate::{
    operation::{PlanError, SolveError},
    response::{ErrorCode, GraphqlError},
};

pub(super) type PrepareResult<T> = std::result::Result<T, PrepareError>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum PrepareError {
    #[error("{0}")]
    OperationError(#[from] operation::Error),
    #[error("{err}")]
    Solve {
        attributes: Box<Option<GraphqlOperationAttributes>>,
        err: SolveError,
    },
    #[error("{err}")]
    Plan {
        attributes: Box<Option<GraphqlOperationAttributes>>,
        err: PlanError,
    },
    #[error("{err}")]
    ComplexityError {
        err: ComplexityError,
        attributes: Box<Option<OperationAttributes>>,
    },
    #[error("Executable document exceeded the maximum configured size")]
    QueryTooBig,
}

impl From<PrepareError> for GraphqlError {
    fn from(val: PrepareError) -> Self {
        match val {
            PrepareError::OperationError(err) => match err {
                operation::Error::Parsing { message, locations } => {
                    GraphqlError::new(message, ErrorCode::OperationParsingError).with_locations(locations)
                }
                operation::Error::Validation { message, locations, .. } => {
                    GraphqlError::new(message, ErrorCode::OperationValidationError).with_locations(locations)
                }
            },
            PrepareError::ComplexityError { err, .. } => {
                GraphqlError::new(err.to_string(), ErrorCode::OperationValidationError)
            }
            PrepareError::Plan { err, .. } => err.into(),
            PrepareError::Solve { err, .. } => err.into(),
            PrepareError::QueryTooBig => GraphqlError::new(val.to_string(), ErrorCode::OperationValidationError),
        }
    }
}

impl PrepareError {
    pub fn take_operation_attributes(&mut self) -> Option<GraphqlOperationAttributes> {
        match self {
            PrepareError::OperationError(err) => match err {
                operation::Error::Parsing { .. } => None,
                operation::Error::Validation { attributes, .. } => {
                    let OperationAttributes {
                        ty,
                        name,
                        sanitized_query,
                    } = std::mem::replace(
                        attributes,
                        OperationAttributes {
                            ty: grafbase_telemetry::graphql::OperationType::Query,
                            name: grafbase_telemetry::graphql::OperationName::Unknown,
                            sanitized_query: Default::default(),
                        },
                    );
                    Some(GraphqlOperationAttributes {
                        ty,
                        name,
                        sanitized_query,
                        complexity_cost: None,
                    })
                }
            },
            PrepareError::ComplexityError { attributes, .. } => {
                std::mem::take(attributes).map(|attr| GraphqlOperationAttributes {
                    ty: attr.ty,
                    name: attr.name,
                    sanitized_query: attr.sanitized_query,
                    complexity_cost: None,
                })
            }
            PrepareError::Solve { attributes, .. } => std::mem::take(attributes),
            PrepareError::Plan { attributes, .. } => std::mem::take(attributes),
            _ => None,
        }
    }
}
