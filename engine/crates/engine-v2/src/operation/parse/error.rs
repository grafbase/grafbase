use itertools::Itertools;

use crate::{response::GraphqlError, ErrorCode};

use super::{Location, LocationError};

pub(crate) type ParseResult<T> = Result<T, ParseError>;

#[derive(thiserror::Error, Debug)]
pub(crate) enum ParseError {
    #[error("Unknown operation named '{0}'.")]
    UnknowOperation(String),
    #[error("Missing operation name.")]
    MissingOperationName,
    #[error(transparent)]
    ParserError(#[from] engine_parser::Error),
    #[error("Query is too complex.")]
    QueryTooComplex { complexity: usize, location: Location },
    #[error("Query contains too many root fields.")]
    QueryContainsTooManyRootFields { count: usize, location: Location },
    #[error("Query contains too many aliases.")]
    QueryContainsTooManyAliases { count: usize, location: Location },
    #[error("Query is nested too deep.")]
    QueryTooDeep { depth: usize, location: Location },
    #[error("Unknown fragment named '{name}'")]
    UnknownFragment { name: String, location: Location },
    #[error("Fragment cycle detected: {}", .cycle.iter().join(", "))]
    FragmentCycle { cycle: Vec<String>, location: Location },
    #[error("Query is too big: {0}")]
    QueryTooBig(#[from] LocationError),
}

impl From<ParseError> for GraphqlError {
    fn from(err: ParseError) -> Self {
        let message = err.to_string();
        match err {
            ParseError::ParserError(err) => GraphqlError::new(message, ErrorCode::OperationParsingError)
                .with_locations(err.positions().filter_map(|pos| pos.try_into().ok())),
            ParseError::QueryTooBig(_) => GraphqlError::new(message, ErrorCode::OperationParsingError),
            ParseError::UnknowOperation(_) | ParseError::MissingOperationName => {
                GraphqlError::new(message, ErrorCode::OperationValidationError)
            }
            ParseError::QueryTooComplex { location, .. }
            | ParseError::QueryContainsTooManyRootFields { location, .. }
            | ParseError::QueryContainsTooManyAliases { location, .. }
            | ParseError::QueryTooDeep { location, .. }
            | ParseError::UnknownFragment { location, .. }
            | ParseError::FragmentCycle { location, .. } => {
                GraphqlError::new(message, ErrorCode::OperationValidationError).with_locations([location])
            }
        }
    }
}
