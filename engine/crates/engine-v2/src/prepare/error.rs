use grafbase_telemetry::graphql::GraphqlOperationAttributes;

use crate::{
    operation::{BindError, ParseError},
    plan::PlanError,
    response::{ErrorCode, GraphqlError},
};

pub(super) type PrepareResult<T> = std::result::Result<T, PrepareError>;

#[derive(Debug, thiserror::Error)]
pub(super) enum PrepareError {
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error("{err}")]
    Bind {
        attributes: Box<Option<GraphqlOperationAttributes>>,
        err: BindError,
    },
    #[error("{err}")]
    Plan {
        attributes: Box<Option<GraphqlOperationAttributes>>,
        err: PlanError,
    },
    #[error("Failed to normalize query")]
    NormalizationError,
}

impl From<PrepareError> for GraphqlError {
    fn from(err: PrepareError) -> Self {
        match err {
            PrepareError::Bind { err, .. } => err.into(),
            PrepareError::Parse(err) => err.into(),
            PrepareError::Plan { err, .. } => err.into(),
            PrepareError::NormalizationError => GraphqlError::new(err.to_string(), ErrorCode::InternalServerError),
        }
    }
}

impl PrepareError {
    pub fn take_operation_attributes(&mut self) -> Option<GraphqlOperationAttributes> {
        match self {
            PrepareError::Bind { attributes, .. } => std::mem::take(attributes),
            PrepareError::Plan { attributes, .. } => std::mem::take(attributes),
            _ => None,
        }
    }
}
