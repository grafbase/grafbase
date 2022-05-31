use std::{io, process::Output, thread};

use crate::{errors::LocalGatewayError, utils::get_availble_port};
use common::consts::DEFAULT_PORT;

/// starts the dev server if an available port can be found
pub fn start_dev_server(
    external_port: Option<u16>,
    search: bool,
) -> Result<(u16, thread::JoinHandle<Result<Output, io::Error>>), LocalGatewayError> {
    let start_port = external_port.unwrap_or(DEFAULT_PORT);
    match get_availble_port(search, start_port) {
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
