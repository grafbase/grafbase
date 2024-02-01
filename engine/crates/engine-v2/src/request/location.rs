use std::fmt;

use super::bind::BindError;

// 65 KB for query without any new lines is pretty huge. If a user ever has a QueryTooBig error
// we'll increase it to u32. But for now it's just wasted memory.
#[derive(Debug, PartialEq, Eq, Clone, Copy, serde::Serialize)]
pub struct Location {
    /// One-based line number.
    pub line: u16,
    /// One-based column number.
    pub column: u16,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl TryFrom<engine_parser::Pos> for Location {
    type Error = BindError;

    fn try_from(value: engine_parser::Pos) -> Result<Self, Self::Error> {
        Ok(Self {
            line: value
                .line
                .try_into()
                .map_err(|_| BindError::QueryTooBig(format!("Too many lines ({})", value.line)))?,
            column: value
                .column
                .try_into()
                .map_err(|_| BindError::QueryTooBig(format!("Too many columns ({})", value.column)))?,
        })
    }
}
