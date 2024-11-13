use std::fmt;

// 65 KB for query without any new lines is pretty huge. If a user ever has a QueryTooBig error
// we'll increase it to u32. But for now it's just wasted memory.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub(crate) struct Location {
    /// One-based line number.
    line: u16,
    /// One-based column number.
    column: u16,
}

impl Location {
    // Might want to change Location to be a NonZeroU32 later to optimize Option<Location>
    pub fn new(line: u16, column: u16) -> Self {
        Self { line, column }
    }

    pub fn line(&self) -> u16 {
        self.line
    }

    pub fn column(&self) -> u16 {
        self.column
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line(), self.column())
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum LocationError {
    #[error("Too many lines ({0})")]
    TooManyLines(usize),
    #[error("Too many columns ({0})")]
    TooManyColumns(usize),
}

impl TryFrom<engine_parser::Pos> for Location {
    type Error = LocationError;

    fn try_from(value: engine_parser::Pos) -> Result<Self, Self::Error> {
        Ok(Self::new(
            value
                .line
                .try_into()
                .map_err(|_| LocationError::TooManyLines(value.line))?,
            value
                .column
                .try_into()
                .map_err(|_| LocationError::TooManyColumns(value.column))?,
        ))
    }
}
