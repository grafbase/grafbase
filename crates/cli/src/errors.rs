use common::{errors::CommonError, traits::ToExitCode};
use local_gateway::errors::{DevServerError, LocalGatewayError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    /// returned if a the shell passed to completions is unsupported or unrecognized
    #[error("received an unknown or unsupported shell for completion generation: {0}")]
    UnsupportedShellForCompletions(String),
    // TODO: this might be better as `expect`
    /// returned if the development server panics
    #[error("{0}")]
    DevServerPanic(String),
    /// wraps a dev server error
    #[error(transparent)]
    DevServerError(DevServerError),
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
            Self::UnsupportedShellForCompletions(_) => exitcode::USAGE,
            Self::DevServerPanic(_) | Self::DevServerError(_) => exitcode::SOFTWARE,
            Self::LocalGatewayError(inner) => inner.to_exit_code(),
            Self::CommonError(inner) => inner.to_exit_code(),
        }
    }
}

impl CliError {
    /// returns the appropriate hint for a [`CliError`]
    pub fn to_hint(&self) -> Option<String> {
        match self {
            Self::LocalGatewayError(LocalGatewayError::AvailablePort) => {
                Some("try supplying a larger port range to search by supplying a lower --port number".to_owned())
            }
            Self::LocalGatewayError(LocalGatewayError::PortInUse(_)) => {
                Some("try using a different --port number or supplying the --search flag".to_owned())
            }
            Self::CommonError(CommonError::FindGrafbaseDirectory) => {
                Some("try running the CLI in your Grafbase project or any nested directory".to_owned())
            }
            Self::DevServerError(DevServerError::NodeInPath) => {
                Some("we currently require Node.js as a dependency - please install Node.js and make sure it is in your $PATH to continue (via installer: https://nodejs.org/en/download/, via package manager: https://nodejs.org/en/download/package-manager/)".to_owned())
            }
            Self::DevServerError(DevServerError::OutdatedNode(_, min_version)) => {
                Some(format!("please update your Node.js version to {min_version} or higher to continue (via installer: https://nodejs.org/en/download/, via package manager: https://nodejs.org/en/download/package-manager/)"))
            }
            _ => None,
        }
    }
}
