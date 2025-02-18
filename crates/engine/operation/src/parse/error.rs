use crate::Error;

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
}

impl ParseError {
    pub fn into_graphql_error(self, offsets: &LineOffsets) -> Error {
        let message = self.to_string();
        match self {
            ParseError::ParserError(err) => {
                let error = Error::parsing(message);
                if let Some(span) = err.span() {
                    error.with_locations(offsets.span_to_location(span))
                } else {
                    error
                }
            }
            ParseError::UnknowOperation(_) | ParseError::MissingOperationName | ParseError::MissingOperations => {
                Error::parsing(message)
            }
        }
    }
}
