use crate::errors::LocalGatewayError;
use common::consts::DEFAULT_PORT;
use common::utils::find_available_port;
use dev_server::errors::DevServerError;
use std::{process::Output, thread};

/// starts the dev server if an available port can be found
///
/// # Errors
///
/// returns [`LocalGatewayError::AvailablePort`] if no available port can  be found
///
/// returns [`LocalGatewayError::PortInUse`] if search is off and the supplied port is in use
pub fn start_dev_server(
    external_port: Option<u16>,
    search: bool,
) -> Result<(u16, thread::JoinHandle<Result<Output, DevServerError>>), LocalGatewayError> {
    let start_port = external_port.unwrap_or(DEFAULT_PORT);
    match find_available_port(search, start_port) {
        Some(port) => match dev_server::start(port) {
            Ok(handle) => Ok((port, handle)),
            Err(error) => Err(LocalGatewayError::DevServerError(error)),
        },
        None => {
            if search {
                Err(LocalGatewayError::AvailablePort)
            } else {
                Err(LocalGatewayError::PortInUse(start_port))
            }
        }
    }
}
