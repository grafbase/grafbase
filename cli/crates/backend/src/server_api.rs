use crate::errors::BackendError;
use crate::types::ServerMessage;
use common::utils::find_available_port;
use common::{consts::DEFAULT_PORT, types::LocalAddressType};
use server::errors::ServerError;
use std::sync::mpsc::Receiver;
use std::thread;

type ServerInfo = (thread::JoinHandle<Result<(), ServerError>>, Receiver<ServerMessage>);

/// starts the server if an available port can be found
///
/// # Errors
///
/// returns [`BackendError::AvailablePort`] if no available port can  be found
///
/// returns [`BackendError::PortInUse`] if search is off and the supplied port is in use
pub fn start_server(external_port: Option<u16>, search: bool, watch: bool) -> Result<ServerInfo, BackendError> {
    let start_port = external_port.unwrap_or(DEFAULT_PORT);
    match find_available_port(search, start_port, LocalAddressType::Localhost) {
        Some(port) => {
            let (handle, receiver) = server::start(port, watch);
            Ok((handle, receiver))
        }
        None => {
            if search {
                Err(BackendError::AvailablePort)
            } else {
                Err(BackendError::PortInUse(start_port))
            }
        }
    }
}
