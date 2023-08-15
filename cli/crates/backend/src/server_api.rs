use crate::types::ServerMessage;
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
pub fn start_server(start_port: u16, search: bool, watch: bool, tracing: bool) -> ServerInfo {
    server::start(start_port, search, watch, tracing)
}
