use engine_parser::Error as ParseError;
use std::fmt;

#[derive(Debug)]
pub enum CodegenError {
    ParseError(ParseError),
    FmtError(fmt::Error),
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenError::ParseError(err) => fmt::Display::fmt(err, f),
            CodegenError::FmtError(err) => fmt::Display::fmt(err, f),
        }
    }
}

impl From<fmt::Error> for CodegenError {
    fn from(value: fmt::Error) -> Self {
        CodegenError::FmtError(value)
    }
}

impl From<ParseError> for CodegenError {
    fn from(value: ParseError) -> Self {
        CodegenError::ParseError(value)
    }
}
