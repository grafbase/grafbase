use common::traits::ToExitCode;
use thiserror::Error;

pub use dev_server::errors::DevServerError;

#[derive(Error, Debug)]
pub enum LocalGatewayError {
    /// returned if no port is available.
    /// used specifically when searching for ports
    #[error("could not find an available port")]
    AvailablePort,
    /// returned if a given port is in use and the search option is not used
    #[error("port {0} is currently in use")]
    PortInUse(u16),
    /// wraps a dev server error
    #[error(transparent)]
    DevServerError(DevServerError),
}

impl ToExitCode for LocalGatewayError {
    fn to_exit_code(&self) -> i32 {
        match &self {
            Self::AvailablePort | Self::PortInUse(_) => exitcode::UNAVAILABLE,
            Self::DevServerError(inner) => inner.to_exit_code(),
        }
    }
}
