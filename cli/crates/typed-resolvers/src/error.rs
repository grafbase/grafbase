use std::fmt;

#[derive(thiserror::Error, Debug)]
pub enum CodegenError {
    #[error(transparent)]
    FmtError(#[from] fmt::Error),
}
