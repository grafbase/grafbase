use crate::output::report;
use crate::CliError;
use common::consts::DEFAULT_PORT;
use common::utils::get_thread_panic_message;
use local_gateway::dev_server_api::start_dev_server;
use local_gateway::types::ServerMessage;

/// cli wrapper for [`local_gateway::dev_server_api::start_dev_server`]
///
/// # Errors
///
/// returns [`CliError::LocalGatewayError`] if the the local gateway returns an error
///
/// returns [`CliError::DevServerPanic`] if the development server panics
pub fn dev(search: bool, external_port: Option<u16>) -> Result<(), CliError> {
    trace!("attempting to start dev server");
    let start_port = external_port.unwrap_or(DEFAULT_PORT);
    let server_handle = match start_dev_server(external_port, search) {
        Ok((handle, receiver)) => {
            if let Ok(message) = receiver.recv() {
                match message {
                    ServerMessage::Ready(port) => report::start_server(port, start_port),
                }
            }
            handle
        }
        Err(error) => return Err(CliError::LocalGatewayError(error)),
    };

    server_handle
        .join()
        .map_err(|parameter| match get_thread_panic_message(&parameter) {
            Some(message) => CliError::DevServerPanic(message),
            None => CliError::DevServerPanic("unknown error".to_owned()),
        })?
        .map_err(CliError::DevServerError)?;

    Ok(())
}
