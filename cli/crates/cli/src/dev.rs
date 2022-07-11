use crate::output::report;
use crate::CliError;
use backend::server_api::start_server;
use backend::types::ServerMessage;
use common::consts::DEFAULT_PORT;
use common::environment::Environment;
use common::utils::get_thread_panic_message;

/// cli wrapper for [`backend::server_api::start_server`]
///
/// # Errors
///
/// returns [`CliError::BackendError`] if the the local gateway returns an error
///
/// returns [`CliError::ServerPanic`] if the development server panics
pub fn dev(search: bool, external_port: Option<u16>) -> Result<(), CliError> {
    trace!("attempting to start server");

    Environment::try_init().map_err(CliError::CommonError)?;

    let start_port = external_port.unwrap_or(DEFAULT_PORT);
    let server_handle = match start_server(external_port, search) {
        Ok((handle, receiver)) => {
            if let Ok(message) = receiver.recv() {
                match message {
                    ServerMessage::Ready(port) => report::start_server(port, start_port),
                }
            }
            handle
        }
        Err(error) => return Err(CliError::BackendError(error)),
    };

    server_handle
        .join()
        .map_err(|parameter| match get_thread_panic_message(&parameter) {
            Some(message) => CliError::ServerPanic(message),
            None => CliError::ServerPanic("unknown error".to_owned()),
        })?
        .map_err(CliError::ServerError)?;

    Ok(())
}
