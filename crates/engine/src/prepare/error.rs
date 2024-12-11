use grafbase_telemetry::graphql::GraphqlOperationAttributes;

use crate::{
    operation::{PlanError, SolveError},
    response::{ErrorCode, GraphqlError},
};

pub(super) type PrepareResult<T> = std::result::Result<T, PrepareError>;

#[derive(Debug, thiserror::Error)]
pub(super) enum PrepareError {
    #[error("{0}")]
    Parse(GraphqlError),
    #[error("{err}")]
    Bind {
        attributes: Box<Option<GraphqlOperationAttributes>>,
        err: GraphqlError,
    },
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
    #[error("Query exceeded complexity limit")]
    ComplexityLimitReached,
    #[error("Expected exactly one slicing argument on {0}")]
    ExpectedOneSlicingArgument(String),
    #[error("Executable document exceeded the maximum configured size")]
    QueryTooBig,
}

impl From<PrepareError> for GraphqlError {
    fn from(val: PrepareError) -> Self {
        match val {
            PrepareError::Bind { err, .. } => err,
            PrepareError::Parse(err) => err,
            PrepareError::Plan { err, .. } => err.into(),
            PrepareError::Solve { err, .. } => err.into(),
            PrepareError::ComplexityLimitReached
            | PrepareError::ExpectedOneSlicingArgument(_)
            | PrepareError::QueryTooBig => GraphqlError::new(val.to_string(), ErrorCode::OperationValidationError),
        }
    }
}

impl PrepareError {
    pub fn take_operation_attributes(&mut self) -> Option<GraphqlOperationAttributes> {
        match self {
            PrepareError::Bind { attributes, .. } => std::mem::take(attributes),
            PrepareError::Solve { attributes, .. } => std::mem::take(attributes),
            PrepareError::Plan { attributes, .. } => std::mem::take(attributes),
            _ => None,
        }
    }
}
