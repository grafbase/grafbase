use std::path::PathBuf;

use common::traits::ToExitCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevServerError {
    /// returned if the current directory path cannot be read
    #[error("could not create path '{0}' for the embedded server files")]
    CreateDir(PathBuf),

    /// returned if any of the embedded worker files cannot be written to disk
    #[error("could not write an embedded server file: {0}")]
    WriteFile(String),

    /// returned if the version of the existing worker files cannot be read
    #[error("could not read the previously extracted embedded file versions")]
    ReadVersion,
}

impl ToExitCode for DevServerError {
    fn to_exit_code(&self) -> i32 {
        match &self {
            Self::CreateDir(_) => exitcode::DATAERR,
            Self::WriteFile(_) => exitcode::DATAERR,
            Self::ReadVersion => exitcode::DATAERR,
        }
    }
}
