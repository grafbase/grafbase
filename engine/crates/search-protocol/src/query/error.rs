use serde::{Deserialize, Serialize};

use super::{GraphqlCursor, Range, ScalarValue};

#[derive(thiserror::Error, Debug, PartialEq, Serialize, Deserialize)]
pub enum QueryError {
    #[error("Search request failed.")]
    ServerError,
    #[error(transparent)]
    BadRequestError(#[from] BadRequestError),
}

#[derive(thiserror::Error, Debug, PartialEq, Serialize, Deserialize)]
pub enum BadRequestError {
    #[error("Invalid Cursor: {0:?}")]
    InvalidCursor(GraphqlCursor),
    #[error("Invalid regex '{pattern}': {err}")]
    InvalidRegex { pattern: String, err: String },
    // Shouldn't happen with proper GraphQL validation.
    #[error("Incompatible ranges: {a} and {b}")]
    IncompatibleRanges {
        a: Box<Range<ScalarValue>>,
        b: Box<Range<ScalarValue>>,
    },
}
