use common::{errors::CommonError, traits::ToExitCode};
use local_gateway::errors::LocalGatewayError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    /// returned if a user supplied port is cannot be parsed
    #[error("could not parse the supplied port")]
    ParsePort,
    /// returned if a the shell passed to completions is unsupported or unrecognized
    #[error("received an unknown or unsupported shell for completion generation: {0}")]
    UnsupportedShellForCompletions(String),
    /// returned if the development server panics
    #[error("the development server panicked: {0}")]
    DevServerPanic(String),
    /// wraps an error originating in the local-gateway crate
    #[error(transparent)]
    LocalGatewayError(LocalGatewayError),
    /// wraps an error originating in the common crate
    #[error(transparent)]
    CommonError(CommonError),
}

impl ToExitCode for CliError {
    fn to_exit_code(&self) -> i32 {
        match &self {
            CliError::ParsePort => exitcode::USAGE,
            CliError::UnsupportedShellForCompletions(_) => exitcode::USAGE,
            CliError::DevServerPanic(_) => exitcode::SOFTWARE,
            CliError::LocalGatewayError(inner) => inner.to_exit_code(),
            CliError::CommonError(inner) => inner.to_exit_code(),
        }
    }
}
