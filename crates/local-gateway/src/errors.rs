use common::traits::ToExitCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LocalGatewayError {
    /// returned if no port is available.
    /// used specifically when searching for ports
    #[error("could not find an available port")]
    AvailablePort,
    /// returned if a given port is in use and the search option is not used
    #[error("port {0} in use")]
    PortInUse(u16),
}

impl ToExitCode for LocalGatewayError {
    fn to_exit_code(&self) -> i32 {
        match &self {
            LocalGatewayError::AvailablePort => exitcode::UNAVAILABLE,
            LocalGatewayError::PortInUse(_) => exitcode::UNAVAILABLE,
        }
    }
}
