use crate::traits::ToExitCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommonError {
    /// returned if the current directory path cannot be read
    #[error("could not read the current path")]
    ReadCurrentDirectory,
    /// returned if the grafbase directory cannot be found
    #[error("could not find the grafbase directory")]
    FindGrafbaseDirectory,
    /// returned if the static environment object cannot be set.
    /// likely to be a bug
    #[error("could not set the environment, this is likely a bug")]
    SetEnvironment,
}

impl ToExitCode for CommonError {
    fn to_exit_code(&self) -> i32 {
        match &self {
            CommonError::ReadCurrentDirectory => exitcode::DATAERR,
            CommonError::FindGrafbaseDirectory => exitcode::DATAERR,
            CommonError::SetEnvironment => exitcode::SOFTWARE,
        }
    }
}
