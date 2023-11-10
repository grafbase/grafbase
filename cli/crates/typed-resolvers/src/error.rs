use engine_parser::Error as ParseError;
use std::fmt;

#[derive(thiserror::Error, Debug)]
pub enum CodegenError {
    #[error("The `codegen` experimental feature is not enabled")]
    ExperimentalFeatureNotEnabled,
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    FmtError(#[from] fmt::Error),
}
