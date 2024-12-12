use std::fmt;

// 65 KB for query without any new lines is pretty huge. If a user ever has a QueryTooBig error
// we'll increase it to u32. But for now it's just wasted memory.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Location {
    /// One-based line number.
    pub line: u16,
    /// One-based column number.
    pub column: u16,
}

impl Location {
    pub fn new(line: u16, column: u16) -> Self {
        Self { line, column }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}
