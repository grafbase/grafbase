use cynic_parser::Span;
use itertools::Itertools;

use crate::{response::GraphqlError, ErrorCode};

use super::offsets::LineOffsets;

pub(super) type ParseResult<T> = Result<T, ParseError>;

#[derive(thiserror::Error, Debug, Clone)]
pub(super) enum ParseError {
    #[error("Unknown operation named '{0}'.")]
    UnknowOperation(String),
    #[error("Missing operation name.")]
    MissingOperationName,
    #[error("The document does not contain any operations")]
    MissingOperations,
    #[error(transparent)]
    ParserError(#[from] cynic_parser::Error),
    #[error("Query is too complex.")]
    QueryTooComplex { complexity: usize, span: Span },
    #[error("Query contains too many root fields.")]
    QueryContainsTooManyRootFields { count: usize, span: Span },
    #[error("Query contains too many aliases.")]
    QueryContainsTooManyAliases { count: usize, span: Span },
    #[error("Query is nested too deep.")]
    QueryTooDeep { depth: usize, span: Span },
    #[error("Unknown fragment named '{name}'")]
    UnknownFragment { name: String, span: Span },
    #[error("Fragment cycle detected: {}", .cycle.iter().join(", "))]
    FragmentCycle { cycle: Vec<String>, span: Span },
}

impl ParseError {
    pub fn into_graphql_error(self, offsets: &LineOffsets) -> GraphqlError {
        let message = self.to_string();
        match self {
            ParseError::ParserError(err) => GraphqlError::new(message, ErrorCode::OperationParsingError)
                .with_locations(offsets.span_to_location(err.span())),
            ParseError::UnknowOperation(_) | ParseError::MissingOperationName | ParseError::MissingOperations => {
                GraphqlError::new(message, ErrorCode::OperationValidationError)
            }
            ParseError::QueryTooComplex { span: location, .. }
            | ParseError::QueryContainsTooManyRootFields { span: location, .. }
            | ParseError::QueryContainsTooManyAliases { span: location, .. }
            | ParseError::QueryTooDeep { span: location, .. }
            | ParseError::UnknownFragment { span: location, .. }
            | ParseError::FragmentCycle { span: location, .. } => {
                GraphqlError::new(message, ErrorCode::OperationValidationError)
                    .with_locations(offsets.span_to_location(location))
            }
        }
    }
}
