use std::thread;

use crate::{errors::LocalGatewayError, utils::get_availble_port};
use common::consts::DEFAULT_PORT;

/// starts the dev server if an available port can be found
pub fn start_dev_server(
    external_port: Option<u16>,
    search: bool,
) -> Result<(u16, thread::JoinHandle<()>), LocalGatewayError> {
    let start_port = external_port.unwrap_or(DEFAULT_PORT);
    match get_availble_port(search, start_port) {
        Some(port) => {
            let handle = dev_server::start(port);
            Ok((port, handle))
        }
        None => {
            if search {
                Err(LocalGatewayError::AvailablePort)
            } else {
                Err(LocalGatewayError::PortInUse(start_port))
            }
        }
    }
}
